use colored::Colorize;
use itertools::Itertools;

use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
// use super::SuiMoveValue;
use serde_with::serde_as;
use std::collections::BTreeMap;

use crate::ic_sui::sui_types::sui_serde::SuiStructTag;
use crate::ic_sui::{
    move_core_types::language_storage::StructTag,
    sui_types::base_types::{ObjectID, SuiAddress},
};
use std::fmt;
use std::fmt::{Display, Formatter, Write};
// use sui_macros::EnumVariantOrder;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged, rename = "MoveValue")]
pub enum SuiMoveValue {
    // u64 and u128 are converted to String to avoid overflow
    Number(u32),
    Bool(bool),
    Address(SuiAddress),
    Vector(Vec<SuiMoveValue>),
    String(String),
    UID { id: ObjectID },
    Struct(SuiMoveStruct),
    Option(Box<Option<SuiMoveValue>>),
    Variant(SuiMoveVariant),
}

impl Display for SuiMoveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match self {
            SuiMoveValue::Number(value) => write!(writer, "{}", value)?,
            SuiMoveValue::Bool(value) => write!(writer, "{}", value)?,
            SuiMoveValue::Address(value) => write!(writer, "{}", value)?,
            SuiMoveValue::String(value) => write!(writer, "{}", value)?,
            SuiMoveValue::UID { id } => write!(writer, "{id}")?,
            SuiMoveValue::Struct(value) => write!(writer, "{}", value)?,
            SuiMoveValue::Option(value) => write!(writer, "{:?}", value)?,
            SuiMoveValue::Vector(vec) => {
                write!(
                    writer,
                    "{}",
                    vec.iter().map(|value| format!("{value}")).join(",\n")
                )?;
            }
            SuiMoveValue::Variant(value) => write!(writer, "{}", value)?,
        }
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "MoveVariant")]
pub struct SuiMoveVariant {
    #[serde(rename = "type")]
    #[serde_as(as = "SuiStructTag")]
    pub type_: StructTag,
    pub variant: String,
    pub fields: BTreeMap<String, SuiMoveValue>,
}

impl Display for SuiMoveVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        let SuiMoveVariant {
            type_,
            variant,
            fields,
        } = self;
        writeln!(writer)?;
        writeln!(writer, "  {}: {type_}", "type".bold().bright_black())?;
        writeln!(writer, "  {}: {variant}", "variant".bold().bright_black())?;
        for (name, value) in fields {
            let value = format!("{}", value);
            let value = if value.starts_with('\n') {
                indent(&value, 2)
            } else {
                value
            };
            writeln!(writer, "  {}: {value}", name.bold().bright_black())?;
        }

        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged, rename = "MoveStruct")]
pub enum SuiMoveStruct {
    Runtime(Vec<SuiMoveValue>),
    WithTypes {
        #[serde(rename = "type")]
        #[serde_as(as = "SuiStructTag")]
        type_: StructTag,
        fields: BTreeMap<String, SuiMoveValue>,
    },
    WithFields(BTreeMap<String, SuiMoveValue>),
}

impl Display for SuiMoveStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match self {
            SuiMoveStruct::Runtime(_) => {}
            SuiMoveStruct::WithFields(fields) => {
                for (name, value) in fields {
                    writeln!(writer, "{}: {value}", name.bold().bright_black())?;
                }
            }
            SuiMoveStruct::WithTypes { type_, fields } => {
                writeln!(writer)?;
                writeln!(writer, "  {}: {type_}", "type".bold().bright_black())?;
                for (name, value) in fields {
                    let value = format!("{}", value);
                    let value = if value.starts_with('\n') {
                        indent(&value, 2)
                    } else {
                        value
                    };
                    writeln!(writer, "  {}: {value}", name.bold().bright_black())?;
                }
            }
        }
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

fn indent<T: Display>(d: &T, indent: usize) -> String {
    d.to_string()
        .lines()
        .map(|line| format!("{:indent$}{}", "", line))
        .join("\n")
}
