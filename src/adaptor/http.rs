const ID: &str = "http";
mod client;
mod server;
pub struct Client {}

pub struct Http {
    client: reqwest::Client,
}

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::task::JoinHandle;

use crate::{
    error::{Error, ErrorCode},
    model::{Message, Receipt},
    node, Node, node_opt,
};

use self::server::Server;

use super::{AdaptorServer, AdaptorSpecifier, AdaptorClient};


impl Http {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::default(),
            // server: Server::start(),
        }
    }
}




impl AdaptorClient for Http {
    fn spec(&self) -> AdaptorSpecifier {
        AdaptorSpecifier::new(ID)
    }

    fn post_message(&self, message: &Message, addr: &str) -> crate::Result<()> {
        use futures::FutureExt;
        use nanoid::nanoid;
        let mut url = reqwest::Url::parse(addr).map_err(Error::code(ErrorCode::Invalid))?;
        url.set_path("message");
        let message_id = message.id.clone();
        let _task_handle = tokio::spawn(
            self.client
                .post(url)
                .json(message)
                .send()
                .then(|result| async move {
                    match result {
                        Ok(resp) => resp.json::<Receipt>().await.unwrap_or(Receipt {
                            id: nanoid!(),
                            message_id,
                            trace: crate::model::ReceiptTrace::Fail {
                                reason: "parse error".to_string(),
                            },
                        }),
                        Err(e) => Receipt {
                            id: nanoid!(),
                            message_id,
                            trace: crate::model::ReceiptTrace::Fail {
                                reason: e.to_string(),
                            },
                        },
                    }
                })
                .then(|receipt| async move { let node = node_opt()?; node.database.save_receipt(&receipt) }),
        );
        Ok(())
    }
    // fn recv_stream(&self) -> std::pin::Pin<Box<dyn futures::Stream<Item = Message> + Send>> {
    //     let chan =
    //     todo!()
    // }
}
