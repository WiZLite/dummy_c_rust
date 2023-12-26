use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum};

use super::*;
use crate::{ast::BinaryOp, resolved_ast::*};

impl LLVMCodeGenerator<'_> {
    fn eval_u8(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u8>().unwrap();
        let int_value = self.llvm_context.i8_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_i32(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<i32>().unwrap();
        let int_value = self.llvm_context.i32_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_i64(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<i64>().unwrap();
        let int_value = self.llvm_context.i64_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_u32(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u32>().unwrap();
        let int_value = self.llvm_context.i32_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_u64(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u64>().unwrap();
        let int_value = self.llvm_context.i64_type().const_int(n, true);
        int_value.into()
    }
    fn eval_usize(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<usize>().unwrap();
        let int_value = self.ptr_sized_int_type.const_int(n as u64, true);
        int_value.into()
    }
    fn eval_number_literal(
        &self,
        integer_literal: &NumberLiteral,
        ty: &ResolvedType,
    ) -> BasicValueEnum {
        let value_str = &integer_literal.value;
        match ty {
            ResolvedType::U8 => self.eval_u8(value_str),
            ResolvedType::U32 => self.eval_u32(value_str),
            ResolvedType::I32 => self.eval_i32(value_str),
            ResolvedType::I64 => self.eval_i64(value_str),
            ResolvedType::U64 => self.eval_u64(value_str),
            ResolvedType::USize => self.eval_usize(value_str),
            ResolvedType::Ptr(_) => unreachable!(),
            ResolvedType::Void => unreachable!(),
            ResolvedType::Unknown => unreachable!(),
        }
    }
    fn eval_string_literal(&self, string_literal: &StringLiteral) -> BasicValueEnum {
        let value = string_literal.value.as_str();
        let string = self
            .llvm_builder
            .build_global_string_ptr(value, "string_literal");
        string.as_basic_value_enum()
    }
    fn eval_variable_ref(&self, variable_ref: &VariableRefExpr) -> BasicValueEnum {
        let ptr = self.get_variable(&variable_ref.name);
        let value = self.llvm_builder.build_load(ptr, "load");
        value
    }
    fn eval_index_access(&self, index_access: &IndexAccessExor) -> BasicValueEnum {
        let ptr = self.gen_expression(&index_access.target).unwrap();
        let index = self.gen_expression(&index_access.index).unwrap();
        let ptr = unsafe {
            self.llvm_builder.build_gep(
                ptr.into_pointer_value(),
                &[index.into_int_value()],
                "index_access",
            )
        };
        let value = self.llvm_builder.build_load(ptr, "load");
        value
    }
    fn eval_deref(&self, deref: &DerefExpr) -> BasicValueEnum {
        let ptr = self.gen_expression(&deref.target).unwrap();
        let value = self
            .llvm_builder
            .build_load(ptr.into_pointer_value(), "load");
        value
    }
    fn eval_binary_expr(&self, binary_expr: &BinaryExpr) -> BasicValueEnum {
        let mut left = self.gen_expression(&binary_expr.lhs).unwrap();
        let mut right = self.gen_expression(&binary_expr.rhs).unwrap();

        let (lhs_cast_type, rhs_cast_type) =
            self.get_cast_type(&binary_expr.lhs.ty, &binary_expr.rhs.ty);

        let mut result_type = ResolvedType::I32;
        if let Some(lhs_cast_type) = lhs_cast_type {
            left = self.gen_try_cast(left, &lhs_cast_type);
            result_type = lhs_cast_type;
        }
        if let Some(rhs_cast_type) = rhs_cast_type {
            right = self.gen_try_cast(right, &rhs_cast_type);
            result_type = rhs_cast_type;
        };

        let value = match binary_expr.op {
            BinaryOp::Add => {
                if result_type.is_integer_type() {
                    self.llvm_builder.build_int_add(
                        left.into_int_value(),
                        right.into_int_value(),
                        "int+int",
                    )
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
        };

        value.as_basic_value_enum()
    }
    pub(super) fn gen_call_expr(&self, call_expr: &CallExpr) -> Option<BasicValueEnum<'_>> {
        let args = call_expr
            .args
            .iter()
            .map(|arg| self.gen_expression(&arg).unwrap().into())
            .collect::<Vec<BasicMetadataValueEnum>>();
        let function = *self.function_by_name.get(&call_expr.callee).unwrap();
        let func = self.gen_or_get_function(function);
        let value = self.llvm_builder.build_call(
            func,
            &args,
            format!("call {}", function.decl.name).as_str(),
        );

        match value.try_as_basic_value().left() {
            Some(value) => Some(value),
            None => None,
        }
    }
    pub(super) fn gen_expression(&self, expr: &ResolvedExpression) -> Option<BasicValueEnum> {
        match &expr.kind {
            ExpressionKind::NumberLiteral(number_literal) => {
                Some(self.eval_number_literal(number_literal, &expr.ty))
            }
            ExpressionKind::VariableRef(variable_ref) => Some(self.eval_variable_ref(variable_ref)),
            ExpressionKind::IndexAccess(index_access) => Some(self.eval_index_access(index_access)),
            ExpressionKind::Deref(deref) => Some(self.eval_deref(deref)),
            ExpressionKind::BinaryExpr(binary_expr) => Some(self.eval_binary_expr(binary_expr)),
            ExpressionKind::CallExpr(call_expr) => self.gen_call_expr(call_expr),
            ExpressionKind::StringLiteral(string_literal) => {
                Some(self.eval_string_literal(string_literal))
            }
        }
    }
}
