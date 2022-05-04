use omnipaxos_core::{
    ballot_leader_election::messages::BLEMessage, messages::Message,
};
use serde::{Deserialize, Serialize};

use super::{kv::{KVSnapshot, KeyValue}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Operaiton {
    Read,
    Write,
    Snap,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CMDMessage {
    pub operation: Operaiton,
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
