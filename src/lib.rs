#[cfg(feature = "scrape")]
pub mod scrape;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub keywords: Vec<String>,
    pub attributes: BTreeMap<String, Attribute>,
    pub builtin_values: BTreeMap<String, BuiltinValue>,
    pub interpolation_type_names: Vec<String>,
    pub interpolation_sampling_names: Vec<String>,
    pub primitive_types: Vec<String>,
    pub type_generators: Vec<String>,
    pub type_aliases: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub description: String,
    pub description_parameters: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltinValue {
    pub stages: BTreeMap<String, BuiltinValueStage>,
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltinValueStage {
    pub description: String,
    pub direction: BuiltinValueDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BuiltinValueDirection {
    Input,
    Output,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FunctionInfo {
    pub functions: BTreeMap<String, Function>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub overloads: Vec<FunctionOverload>,
    pub parameters: Vec<FunctionParameter>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionOverload {
    pub signature: String,
    pub parameterization: Parameterization,
    pub description: Option<String>,
}

/// Dsecribes the valid values for a set of variables.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Parameterization {
    pub typevars: BTreeMap<String, ParameterizationKind>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterizationKind {
    /// A list of types.
    Types(Vec<String>),
    /// A human-readable description.
    Description(String),
}

#[cfg(feature = "include")]
pub mod include {
    pub const TOKENS_JSON: &str = include_str!("../spec/tokens.json");
    pub const FUNCTIONS_JSON: &str = include_str!("../spec/functions.json");

    pub fn tokens() -> serde_json::Result<crate::TokenInfo> {
        serde_json::from_slice(TOKENS_JSON.as_bytes())
    }

    pub fn functions() -> serde_json::Result<crate::FunctionInfo> {
        serde_json::from_slice(FUNCTIONS_JSON.as_bytes())
    }
}
