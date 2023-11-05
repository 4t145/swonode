use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paged<T> {
    data: Vec<T>,
    page: Page,
    total: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Page {
    index: usize,
    size: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct After<'s> {
    key: Cow<'s, str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "policy", content = "data")]
pub enum QueryPolicy<'q> {
    After(After<'q>),
    Page(Page),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Queried<T> {
    pub data: Vec<T>,
    pub remain: usize,
}
