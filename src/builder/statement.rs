use inkwell::{
    builder::BuilderError,
    types::BasicType,
    values::{BasicValue, InstructionValue},
};

use super::*;
use crate::concrete_ast::*;

impl LLVMCodeGenerator<'_> {
    pub(super) fn gen_return(&mut self, ret: &Return) -> Result<InstructionValue, BuilderError> {
        if let Some(expression) = &ret.expression {
            let value = self.gen_expression(expression)?.unwrap();
            let ptr = self.llvm_builder.build_alloca(value.get_type(), "")?;
            if value.is_struct_value() {
                dbg!("value is struct type");
                self.llvm_builder.build_call(
                    self.llvm_module.get_function("memcpy").unwrap(),
                    &[value
                        .get_type()
                        .size_of()
                        .unwrap()
                        .as_basic_value_enum()
                        .into()],
                    "memcpy",
                )?;
            } else {
                self.llvm_builder.build_store(ptr, value)?;
            }
            self.llvm_builder.build_return(Some(&value))
        } else {
            self.llvm_builder.build_return(None)
        }
    }
    pub(super) fn gen_effect(&self, effect: &Effect) -> Result<(), BuilderError> {
        self.gen_expression(&effect.expression)?;
        Ok(())
    }
    pub(super) fn gen_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<Option<InstructionValue>, BuilderError> {
        match &statement {
            Statement::Return(ret) => self.gen_return(ret).map(Some),
            Statement::Effect(effect) => {
                self.gen_effect(effect)?;
                Ok(None)
            }
        }
    }
}
