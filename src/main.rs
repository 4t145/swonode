#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
use std::sync::{Arc, OnceLock};

use adaptor::{AdaptorRouter, AdaptorSpecifier};
use chrono::Utc;
use consts::CHANNEL_BUFFER_SIZE;
use db::{mem::MemDb, Database};
use model::{Agent, EndPoint, Message, Receipt};

mod adaptor;
mod consts;
mod db;
mod error;
mod model;
mod query;
use error::*;
use nanoid::nanoid;

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    tracing_subscriber::fmt::init();

    let endpoint = EndPoint::new(EndPoint::gen_eid());
    let http = adaptor::http::Http::new();
    let mut node = Node::new(endpoint, MemDb::default());
    node.adaptors.register(http);
    NODE.get_or_init(|| node);
    // wait stop signal
    tokio::signal::ctrl_c().await;
    // node.adaptors.register(adaptor)
    Ok(())
}

static NODE: OnceLock<Node> = OnceLock::new();
pub fn node_opt() -> crate::Result<&'static Node> {
    NODE.get().ok_or(Error::from_message(
        "node is not ready yet",
        ErrorCode::Unready,
    ))
}
pub fn node() -> &'static Node {
    NODE.get().expect("not is not ready yet")
}
pub struct Node {
    pub endpoint: EndPoint,
    pub trust: Vec<EndPoint>,
    pub adaptors: AdaptorRouter,
    pub database: Arc<dyn Database>,
}

pub struct NodeBuilder {
    endpoint: EndPoint,
    trust: Vec<EndPoint>,

}


impl Node {
    pub fn new(ep: EndPoint, db: impl Database) -> Self {
        let db = Arc::new(db);
        Self {
            endpoint: ep,
            database: db,
            trust: Default::default(),
            adaptors: Default::default(),
        }
    }
    pub fn endpoint(&self) -> &EndPoint {
        &self.endpoint
    }
    pub fn as_agent(&self) -> Agent {
        Agent {
            eid: self.endpoint.eid.clone(),
            recieve_time: Utc::now(),
            forward_time: Utc::now(),
            meta: Default::default(),
        }
    }

    pub fn send_to_ep(&self, message: &Message, endpoint: &EndPoint) -> Result<Receipt> {
        let mut err_collect = vec![];

        for (spec, addr) in self.endpoint.accept.iter() {
            if endpoint.accept.contains_key(spec) {
                match self.send(message, spec, addr) {
                    Err(e) => {
                        err_collect.push(e);
                        continue;
                    }
                    _ok => {
                        return Ok(Receipt {
                            id: nanoid!(),
                            message_id: message.id.clone(),
                            trace: model::ReceiptTrace::Forward {
                                endpoint: endpoint.clone(),
                                spec: spec.clone(),
                            },
                        })
                    }
                }
            }
        }
        Err(Error {
            message: "All endpoint is unreachable".into(),
            code: ErrorCode::Unreachable,
            traceback: err_collect,
        })
    }

    pub fn send(&self, message: &Message, spec: &AdaptorSpecifier, addr: &str) -> Result<()> {
        let a = self.adaptors.get_client(spec).ok_or(Error::from_message(
            format!("adaptor {} not found", spec),
            ErrorCode::NotFound,
        ))?;
        a.post_message(message, addr)
    }

    pub fn revieve_tome(&self, message: &Message) -> Result<Receipt> {
        todo!("msg to me {message:?}")
    }
    pub fn recieve(&self, message: &Message) -> Result<Receipt> {
        if message.to.eid == self.endpoint.eid {
            self.revieve_tome(message)
        } else {
            self.forward(message)
        }
    }
    fn has_recieved(&self, id: &str) -> bool {
        todo!("")
    }
    fn forward(&self, message: &Message) -> Result<Receipt> {
        // message sand from me should not be processed.
        if self.has_recieved(&message.id)
            || message
                .agents
                .iter()
                .any(|agent| agent.eid == self.endpoint.eid)
        {
            return error(
                format!("This message was sent out from me: {}", self.endpoint.eid),
                ErrorCode::LoopDetected,
            );
        }

        let trust = &self.trust;
        let mut message = message.clone();
        message.agents.push(self.as_agent());
        let mut err_collect = vec![];

        // try to directly send
        match self.send_to_ep(&message, &message.to) {
            Err(e) => err_collect.push(e),
            ok => return ok,
        }

        // forward to other nodes
        for ep in trust
            .iter()
            .filter(|ep| message.agents.iter().all(|agt| agt.eid != ep.eid))
        {
            for spec in ep.accept.keys() {
                if message.to.accept.keys().any(|to_spec| spec == to_spec) {
                    match self.send_to_ep(&message, ep) {
                        Err(e) => err_collect.push(e),
                        ok => return ok,
                    }
                }
            }
        }

        Err(Error {
            message: "all forward and direct send failed".into(),
            code: ErrorCode::Unreachable,
            traceback: err_collect,
        })
    }
}
