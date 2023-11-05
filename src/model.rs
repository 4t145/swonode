use std::{collections::HashMap, borrow::Cow};

use chrono::{DateTime, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{adaptor::AdaptorSpecifier, consts::VER};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct EndPoint {
    pub eid: String,
    pub accept: HashMap<AdaptorSpecifier, String>,
}

impl EndPoint {
    pub fn gen_eid() -> String {
        nanoid!()
    }
    pub fn new(eid: String) -> Self {
        Self {
            eid,
            accept: Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize , Clone)]
#[serde(tag="type", content="content")]
pub enum MessageContent {
    Plain(String),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub swo_version: Cow<'static, str>,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            swo_version: VER.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub eid: String,
    pub recieve_time: DateTime<Utc>,
    pub forward_time: DateTime<Utc>,
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String, 
    pub from: EndPoint,
    pub to: EndPoint,
    pub content: MessageContent,
    pub create_time: DateTime<Utc>,
    pub meta: Meta,
    pub agents: Vec<Agent>,
}


pub struct NodeInfo {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub message_id: String,
    pub trace: ReceiptTrace,
    // pub resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptTrace {
    Forward {
        endpoint: EndPoint,
        spec: AdaptorSpecifier
    },
    Reached {
        endpoint: EndPoint,
    },
    Fail {
        reason: String,
    }
}