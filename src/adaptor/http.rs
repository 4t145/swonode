const ID: &str = "http";

pub struct Client {}

pub struct Server {
    handle: JoinHandle<()>,
}

pub struct Http {
    client: reqwest::Client,
    server: Server,
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

use super::{Adaptor, AdaptorSpecifier};

pub async fn post_message(Json(message): Json<Message>) -> Json<crate::Result<Receipt>> {
    fn post_message_handler(message: Message) -> crate::Result<Receipt> {
        let node = node_opt()?;
        node.recieve(&message)
    }
    Json(post_message_handler(message))
}
impl Http {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::default(),
            server: Server::start(),
        }
    }
}
impl Server {
    pub fn start() -> Server {
        let handle = tokio::spawn(async {
            let app = Router::new()
                // `GET /` goes to `root`
                // .route("/", get(root))
                // `POST /users` goes to `create_user`
                .route("/message", post(post_message));

            // run our app with hyper, listening globally on port 3000
            axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
                .serve(app.into_make_service())
                .await
                .expect("msg");
        });
        Server { handle }
    }
}

impl Adaptor for Http {
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

    fn get_message(&self, page: crate::query::QueryPolicy) -> crate::query::Queried<Message> {
        todo!()
    }

    // fn recv_stream(&self) -> std::pin::Pin<Box<dyn futures::Stream<Item = Message> + Send>> {
    //     let chan =
    //     todo!()
    // }
}
