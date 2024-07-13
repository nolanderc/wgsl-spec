#[cfg(feature = "scrape")]
pub mod scrape;

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TokenInfo {
    pub keywords: Vec<String>,
    pub attributes: Vec<String>,
    pub builtin_values: Vec<String>,
    pub interpolation_type_names: Vec<String>,
    pub interpolation_sampling_names: Vec<String>,
    pub primitive_types: Vec<String>,
    pub type_generators: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct FunctionInfo {
    pub functions: BTreeMap<String, Function>,
}

#[derive(Debug, Serialize)]
pub struct Function {
    pub overloads: Vec<FunctionOverload>,
    pub parameters: Vec<FunctionParameter>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FunctionParameter {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct FunctionOverload {
    pub signature: String,
    pub parameterization: Parameterization,
    pub description: Option<String>,
}

/// Dsecribes the valid values for a set of variables.
#[derive(Debug, Default, Serialize)]
#[serde(transparent)]
pub struct Parameterization {
    pub typevars: BTreeMap<String, ParameterizationKind>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterizationKind {
    /// A list of types.
    Types(Vec<String>),
    /// A human-readable description.
    Description(String),
}
