//! ACBS Package Metadata Language (APML) syntax tree and parsers.

use std::{collections::HashMap, ops::Add};

use ast::{ApmlAst, AstNode};
use lst::ApmlLst;
use thiserror::Error;

pub mod ast;
pub mod eval;
pub mod lst;
pub mod parser;
pub mod pattern;

/// A evaluated APML context.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ApmlContext {
    variables: HashMap<String, VariableValue>,
}

impl ApmlContext {
    /// Evaluates a APML AST, expanding variables.
    pub fn eval_ast(ast: &ApmlAst) -> std::result::Result<Self, ApmlError> {
        let mut apml = ApmlContext::default();
        eval::eval_ast(&mut apml, ast)?;
        Ok(apml)
    }

    /// Emits and evaluates a APML LST.
    pub fn eval_lst(lst: &ApmlLst) -> std::result::Result<Self, ApmlError> {
        Self::eval_ast(&ApmlAst::emit_from(lst)?)
    }

    /// Parses a APML source code, expanding variables.
    pub fn eval_source(src: &str) -> std::result::Result<Self, ApmlError> {
        Self::eval_lst(&ApmlLst::parse(src)?)
    }

    /// Gets a variable value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    /// Gets a variable value or returns a default value if not found.
    #[must_use]
    pub fn read(&self, name: &str) -> VariableValue {
        self.variables.get(name).cloned().unwrap_or_default()
    }

    /// Gets a variable value.
    #[must_use]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut VariableValue> {
        self.variables.get_mut(name)
    }

    /// Removes a variable value.
    pub fn remove(&mut self, name: &str) -> Option<VariableValue> {
        self.variables.remove(name)
    }

    /// Inserts a variable.
    pub fn insert(&mut self, name: String, value: VariableValue) {
        self.variables.insert(name, value);
    }

    /// Iterates over all variable names.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }
}

#[derive(Debug, Error)]
pub enum ApmlError {
    #[error(transparent)]
    Parse(#[from] parser::ParseError),
    #[error(transparent)]
    Emit(#[from] ast::EmitError),
    #[error(transparent)]
    Eval(#[from] eval::EvalError),
}

/// Value of variables.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum VariableValue {
    String(String),
    Array(Vec<String>),
}

impl VariableValue {
    /// Returns the value as an string.
    ///
    /// If the value is an array, it will be converted into a space-delimited
    /// string.
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            VariableValue::String(text) => text.to_owned(),
            VariableValue::Array(els) => els.join(" "),
        }
    }

    /// Returns the value as an string.
    ///
    /// If the value is an array, it will be converted into a space-delimited
    /// string.
    #[must_use]
    pub fn into_string(self) -> String {
        match self {
            VariableValue::String(text) => text,
            VariableValue::Array(els) => els.join(" "),
        }
    }

    /// Returns the value as an array.
    ///
    /// If the value is a string value, it will be converted into a
    /// single-element array. If the string is empty, it will be
    /// converted into a empty array.
    #[must_use]
    pub fn as_array(&self) -> Vec<String> {
        match self {
            VariableValue::String(text) => {
                if text.is_empty() {
                    vec![]
                } else {
                    vec![text.to_owned()]
                }
            }
            VariableValue::Array(els) => els.to_owned(),
        }
    }

    /// Returns the value as an array.
    ///
    /// If the value is a string value, it will be converted into a
    /// single-element array. If the string is empty, it will be
    /// converted into a empty array.
    #[must_use]
    pub fn into_array(self) -> Vec<String> {
        match self {
            VariableValue::String(text) => {
                if text.is_empty() {
                    vec![]
                } else {
                    vec![text]
                }
            }
            VariableValue::Array(els) => els,
        }
    }

    /// Returns the length of string or array.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            VariableValue::String(text) => text.len(),
            VariableValue::Array(els) => els.len(),
        }
    }

    /// Returns if the value is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            VariableValue::String(text) => text.is_empty(),
            VariableValue::Array(els) => els.is_empty(),
        }
    }
}

impl Default for VariableValue {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl Add for VariableValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::String(val) => Self::String(format!("{}{}", val, rhs.into_string())),
            Self::Array(mut val1) => {
                val1.append(&mut rhs.into_array());
                Self::Array(val1)
            }
        }
    }
}

impl<S: AsRef<str>> From<S> for VariableValue {
    fn from(value: S) -> Self {
        Self::String(value.as_ref().to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_variable_value_string() {
        assert_eq!(VariableValue::default().as_string(), "");
        assert_eq!(VariableValue::String("test".into()).as_string(), "test");
        assert_eq!(VariableValue::String("test".into()).into_string(), "test");
        assert_eq!(
            VariableValue::String("".into()).as_array(),
            Vec::<String>::new()
        );
        assert_eq!(VariableValue::String("test".into()).into_array(), vec![
            "test".to_string()
        ]);
        assert_eq!(
            VariableValue::String("".into()).into_array(),
            Vec::<String>::new()
        );
        assert_eq!(VariableValue::String("test".into()).len(), 4);
        assert!(VariableValue::String("".into()).is_empty());
        assert!(!VariableValue::String("test".into()).is_empty());
        assert_eq!(
            VariableValue::String("test".into())
                + "test".into()
                + VariableValue::String("test".into()),
            "testtesttest".into()
        );
    }

    #[test]
    fn test_variable_value_array() {
        assert!(VariableValue::default().as_array().is_empty());
        assert_eq!(
            VariableValue::Array(vec!["test".into()]).as_string(),
            "test"
        );
        assert_eq!(
            VariableValue::Array(vec!["test".into()]).into_string(),
            "test"
        );
        assert_eq!(VariableValue::Array(vec!["test".into()]).as_array(), vec![
            "test".to_string()
        ]);
        assert_eq!(
            VariableValue::Array(vec!["test".into()]).into_array(),
            vec!["test".to_string()]
        );
        assert_eq!(VariableValue::Array(vec!["test".into()]).len(), 1);
        assert!(VariableValue::Array(vec![]).is_empty());
        assert!(!VariableValue::Array(vec!["".into()]).is_empty());
        assert!(!VariableValue::Array(vec!["test".into()]).is_empty());
        assert_eq!(
            VariableValue::Array(vec!["test".into()])
                + VariableValue::Array(vec!["test".into()])
                + VariableValue::Array(vec!["test".into()]),
            VariableValue::Array(vec!["test".into(), "test".into(), "test".into()])
        );
    }

    #[test]
    fn test_apml_parse() {
        let apml = ApmlContext::eval_source(
            r##"# Test APML

PKGVER=8.2
PKGDEP="x11-lib libdrm expat systemd elfutils libvdpau nettle \
        libva wayland s2tc lm-sensors libglvnd llvm-runtime libclc"
MESON_AFTER="-Ddri-drivers-path=/usr/lib/xorg/modules/dri \
             -Db_ndebug=true" 
MESON_AFTER__AMD64=" \
             ${MESON_AFTER} \
             -Dlibunwind=true"
A="${b[@]}"
"##,
        )
        .unwrap();
        dbg!(&apml);
    }
}
