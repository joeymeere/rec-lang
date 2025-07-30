use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct RecDocument {
    pub includes: Vec<String>,
    pub type_definitions: HashMap<String, TypeDef>,
    pub enum_definitions: HashMap<String, EnumDef>,
    pub root: RecObject,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub fields: IndexMap<String, FieldDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub ty: RecType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariant {
    Unit(String),
    Tuple(String, Vec<RecType>),
    Struct(String, IndexMap<String, FieldDef>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecType {
    String,
    Int,
    Float,
    Bool,
    Url,
    Socket,
    Pubkey,
    Array(Box<RecType>),
    Object(String), // named type
    Enum(String),   // enum type
    Any,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RecValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Url(String),
    Socket(String),
    Pubkey(String),
    Array(Vec<RecValue>),
    Object(RecObject),
    EnumVariant {
        enum_name: String,
        variant: String,
        data: EnumVariantData,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<RecValue>),
    Struct(IndexMap<String, RecValue>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecObject {
    pub fields: IndexMap<String, RecValue>,
}
