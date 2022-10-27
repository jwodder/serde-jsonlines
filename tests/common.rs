use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Structure {
    pub name: String,
    pub size: i32,
    pub on: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
pub static DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");
