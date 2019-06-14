
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)]
#[serde(tag = "op")]
pub enum Command {
    Set {
        key: String,
        value: String,
    },
    Remove {
        key: String,
    }
}