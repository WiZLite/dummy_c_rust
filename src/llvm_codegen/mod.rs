mod context;
mod expression;
mod intrinsic;
mod statement;
mod toplevel;
mod value;

use crate::ast::{Module, TopLevel, Type};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::IntType;

use std::cell::RefCell;

use std::rc::Rc;
use thiserror::Error;

use self::context::Context;
use self::value::Value;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Variable `{name:?}` is not found in this scope.")]
    VariableNotFound { name: String },
    #[error("Function `{name:?}` is not found.")]
    FunctionNotFound { name: String },
    #[error("`{name:?}` is not a function")]
    CallNotFunctionValue { name: String },
    #[error("Invalid operand.")]
    InvalidOperand,
    #[error("Invalid operand.")]
    InvalidArgument,
    #[error("Asign value does not match")]
    AsignValueDoesNotMatch {
        expected: Box<Type>,
        actual: Box<Type>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum PointerSize {
    SixteenFour,
}

pub struct LLVMCodegenerator<'a> {
    context: Rc<RefCell<Context<'a>>>,
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
    i8_type: IntType<'a>,
    i32_type: IntType<'a>,
    i64_type: IntType<'a>,
    pointer_size: PointerSize,
}

impl<'a> LLVMCodegenerator<'a> {
    pub fn new(llvm_context: &'a LLVMContext) -> LLVMCodegenerator<'a> {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();
        let i8_type = llvm_context.i8_type();
        let i32_type = llvm_context.i32_type();
        let i64_type = llvm_context.i64_type();
        Self {
            context: Rc::new(RefCell::new(Context::new())),
            llvm_module,
            llvm_builder,
            llvm_context,
            i8_type,
            i32_type,
            i64_type,
            pointer_size: PointerSize::SixteenFour,
        }
    }
}

impl<'a> LLVMCodegenerator<'a> {
    fn get_variable(&self, name: &str) -> Result<Value, CompileError> {
        for scope in self.context.borrow().scopes.iter().rev() {
            if let Some((ty, pointer)) = scope.get(name) {
                let value = self.llvm_builder.build_load(*pointer, name);
                return Ok(match *ty {
                    Type::I32 => Value::I32Value(value.into_int_value()),
                    Type::U8 => Value::U8Value(value.into_int_value()),
                    Type::U32 => Value::U32Value(value.into_int_value()),
                    Type::U64 => Value::U64Value(value.into_int_value()),
                    Type::USize => Value::USizeValue(value.into_int_value()),
                    Type::Ptr(_) => Value::PointerValue(value.into_pointer_value()),
                    Type::Void => Value::Void,
                });
            }
        }
        Err(CompileError::VariableNotFound {
            name: name.to_string(),
        })
    }

    pub fn gen_module(self, module: Module) -> Result<LLVMModule<'a>, CompileError> {
        // Add global scope
        self.context.borrow_mut().push_scope();
        self.context.borrow_mut().push_function_scope();
        self.gen_intrinsic_functions();
        for top in module.toplevels {
            match top {
                TopLevel::Function { decl, body } => self.gen_function(decl, body)?,
            }
        }
        self.context.borrow_mut().pop_scope();
        self.context.borrow_mut().pop_function_scope();
        Ok(self.llvm_module)
    }
}
