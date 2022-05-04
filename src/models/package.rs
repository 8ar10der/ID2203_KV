use serde::{Deserialize, Serialize};

use super::msg::Msg;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Types {
    BLE,
    SP,
    CMD,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Package {
    pub types: Types,
    pub msg: Msg,
}
