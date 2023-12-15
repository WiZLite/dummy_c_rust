use std::fmt::Display;

use thiserror::Error;



#[derive(Debug)]
pub enum ContextType {
    // expressions
    VariableRefExpression,
    CallExpression,
    NumberLiteralExpression,
    BinaryExpression,
    IntrinsicExpression,
    // statements
    AsignStatement,
    ReturnStatement,
    DiscardedExpressionStatement,
    VariableDeclStatement,
    // toplevels
    Function,
}

#[derive(Debug, Error)]
pub enum CompileErrorKind {
    #[error("in {0:?}")]
    Context(ContextType),
    #[error("Variable `{name:?}` is not found in this scope.")]
    VariableNotFound { name: String },
    #[error("Function `{name:?}` is not found.")]
    FunctionNotFound { name: String },
    #[error("`{name:?}` is not a function")]
    IsNotFunction { name: String },
    #[error("`{name:?}` is not a typename")]
    IsNotType { name: String },
    #[error("`{name:?}` is not a variable")]
    IsNotVariable { name: String },
    #[error("Invalid operand.")]
    InvalidOperand(String),
    #[error("Invalid operand.")]
    InvalidArgument,
    #[error("Asign value does not match")]
    TypeMismatch { expected: String, actual: String },
    #[error("Cannot deref {name} for {deref_count:?} times.")]
    CannotDeref { name: String, deref_count: u32 },
    #[error("Cannot access {name} by index.")]
    CannotIndexAccess { name: String, ty: String },
    #[error("Array index must be an integer value")]
    InvalidArrayIndex,
    #[error("Cannot find type name {name}")]
    TypeNotFound { name: String },
    #[error("Too many generic args. Expected {expected:?}, but got {actual:?}")]
    TooManyGenericArgs {
        fn_name: String,
        expected: u32,
        actual: u32,
    },
    #[error("Too few generic args. Expected {expected:?}, but got {actual:?}")]
    TooFewGenericArgs {
        fn_name: String,
        expected: u32,
        actual: u32,
    },
}

#[derive(Debug, Error)]
pub struct CompileError {
    errors: Vec<CompileErrorKind>,
}

#[derive(Debug)]
pub struct FaitalError(pub String);

impl CompileError {
    pub fn from_error_kind(kind: CompileErrorKind) -> Self {
        CompileError { errors: vec![kind] }
    }
    pub fn append(kind: CompileErrorKind, mut other: Self) -> Self {
        other.errors.push(kind);
        other
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! error_context {
    ( $context_type:expr, $generator_block:expr ) => {
        match $generator_block {
            Ok(ret) => Ok(ret),
            Err(compile_error) => Err(self::error::CompileError::append(
                self::error::CompileErrorKind::Context($context_type),
                compile_error,
            )),
        }
    };
}
