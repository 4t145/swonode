pub mod mem;
use crate::{
    model::{Message, Receipt},
    query::{Queried, QueryPolicy},
};

pub struct MessageFilter {}
pub trait Database: Send + Sync + 'static {
    fn save_message(&self, messages: Vec<Message>) -> crate::Result<()>;
    fn save_receipt(&self, receipt: &Receipt) -> crate::Result<()>;
    fn query_message(&self, filter: &MessageFilter, policy: &QueryPolicy) -> Queried<Message>;
}
