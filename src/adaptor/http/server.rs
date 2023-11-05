use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, routing::post, Json, Router};
use tokio::task::JoinHandle;

use crate::{model::*, Node};

pub struct Server {
    handle: JoinHandle<()>,
    host_node: Arc<Node>,
}
#[axum::debug_handler]
pub async fn post_message(
    State(node): State<Arc<Node>>,
    Json(message): Json<Message>,
) -> Json<crate::Result<Receipt>> {
    Json(node.recieve(&message))
}

impl Server {
    pub fn start(&self) {
        let app = Router::new()
            // `GET /` goes to `root`
            // .route("/", get(root))
            // `POST /users` goes to `create_user`
            .route("/message", post(post_message))
            .with_state(self.host_node.clone());
        let handle = tokio::spawn(
            // run our app with hyper, listening globally on port 3000
            axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
                .serve(app.into_make_service()),
        );
    }
}
