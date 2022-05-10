use omnipaxos_core::{
    ballot_leader_election::messages::BLEMessage, messages::Message,
};
use serde::{Deserialize, Serialize};

use super::{kv::{KVSnapshot, KeyValue}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Operation {
    Read,
    Write,
    Snap,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CMDMessage {
    pub operation: Operation,
    pub kv: KeyValue,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Msg {
    BLE(BLEMessage),
    SP(Message<KeyValue, KVSnapshot>),
    CMD(CMDMessage),
}
