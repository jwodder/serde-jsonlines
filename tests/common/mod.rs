use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Structure {
    pub(crate) name: String,
    pub(crate) size: i32,
    pub(crate) on: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[allow(dead_code)]
pub(crate) static DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");
