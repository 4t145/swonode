use super::{Database, MessageFilter};
use crate::{model::*, query::{QueryPolicy, Queried}};
use std::{collections::HashMap, sync::RwLock};

#[derive(Debug, Default)]
pub struct MemDb {
    messages: RwLock<HashMap<String, Message>>,
    receipts: RwLock<HashMap<String, Receipt>>,
}

impl Database for MemDb {
    fn save_message(&self, messages: Vec<Message>) -> crate::Result<()> {
        dbg!(&messages);
        let mut wg = self.messages.write().unwrap();
        for message in messages.into_iter() {
            wg.insert(message.id.clone(), message);
        }
        Ok(())
    }

    fn save_receipt(&self, receipt: &Receipt) -> crate::Result<()> {
        dbg!(&receipt);
        let mut wg = self.receipts.write().unwrap();
        wg.insert(receipt.id.clone(), receipt.clone());
        Ok(())
    }

    fn query_message(
        &self,
        filter: &MessageFilter,
        policy: &QueryPolicy,
    ) -> Queried<crate::model::Message> {
        todo!()
    }
}
