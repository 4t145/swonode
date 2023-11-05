use std::{
    borrow::{BorrowMut, Cow},
    collections::BTreeMap,
    future::Future,
    pin::Pin, sync::Arc,
};

use crate::{
    consts::*,
    model::{EndPoint, Message, Receipt},
    node,
    query::{Queried, QueryPolicy},
    Node,
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

pub mod http;
pub trait Adaptor {
    type Server: AdaptorServer;
    type Client: AdaptorClient;
}
pub trait AdaptorServer: 'static + Send + Sync {
    type Config;
    fn new(config: &Self::Config, host: Arc<Node>) -> Self;
}

pub trait AdaptorClient: 'static + Send + Sync {
    fn spec(&self) -> AdaptorSpecifier;
    fn post_message(&self, message: &Message, addr: &str) -> crate::Result<()>;
}
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct AdaptorSpecifier {
    pub version: Cow<'static, str>,
    pub id: Cow<'static, str>,
}

impl Serialize for AdaptorSpecifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl std::fmt::Display for AdaptorSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id)?;
        f.write_str("@")?;
        f.write_str(&self.version)
    }
}

impl std::str::FromStr for AdaptorSpecifier {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((id, version)) = s.split_once('@') {
            Ok(AdaptorSpecifier {
                id: Cow::Owned(id.to_string()),
                version: Cow::Owned(version.to_string()),
            })
        } else {
            crate::error(
                "invalid adaptor specifier",
                crate::error::ErrorCode::Invalid,
            )
        }
    }
}

impl TryFrom<String> for AdaptorSpecifier {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl AdaptorSpecifier {
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        AdaptorSpecifier {
            version: VER.into(),
            id: id.into(),
        }
    }
}

pub struct AdaptorRouter {
    pub router: BTreeMap<AdaptorSpecifier, WrappedAdaptor>,
    tx: tokio::sync::mpsc::Sender<(AdaptorSpecifier, Message)>,
    rx: tokio::sync::mpsc::Receiver<(AdaptorSpecifier, Message)>,
}

impl Default for AdaptorRouter {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(CHANNEL_BUFFER_SIZE);
        AdaptorRouter {
            router: Default::default(),
            tx,
            rx,
        }
    }
}

pub struct WrappedAdaptor {
    pub adaptor: Box<dyn AdaptorClient>,
    // task_handle: tokio::task::JoinHandle<crate::Result<()>>,
}

impl AdaptorRouter {
    pub fn wrap<A: AdaptorClient>(&self, adaptor: A) -> WrappedAdaptor {
        WrappedAdaptor {
            adaptor: Box::new(adaptor),
            // task_handle,
        }
    }
    pub fn get_client(&self, spec: &AdaptorSpecifier) -> Option<&dyn AdaptorClient> {
        self.router.get(spec).map(|w| w.adaptor.as_ref())
    }
    pub fn register<A: AdaptorClient>(&mut self, adaptor: A) {
        let spec = adaptor.spec();
        let wrapper = self.wrap(adaptor);
        self.router.insert(spec, wrapper);
    }
}
