use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Variable {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Dimension {
    pub name: String,
    pub length: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalAttribute {
    pub name: String,
    pub value: String,
    pub example: String,
    pub compliance: String,
}
