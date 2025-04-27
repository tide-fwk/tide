use crate::CodegenCtx;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use tidec_lir::syntax::LirTy;

pub trait BasicTypesUtils<'ll> {
    fn into_basic_type_metadata(self, ctx: &CodegenCtx<'ll>) -> BasicMetadataTypeEnum<'ll>;
    fn into_basic_type(self, ctx: &CodegenCtx<'ll>) -> BasicTypeEnum<'ll>;
}

impl<'ll> BasicTypesUtils<'ll> for LirTy {
    fn into_basic_type_metadata(self, ctx: &CodegenCtx<'ll>) -> BasicMetadataTypeEnum<'ll> {
        match self {
            LirTy::I8 => BasicTypeEnum::IntType(ctx.ll_context.i8_type()).into(),
            LirTy::I16 => BasicTypeEnum::IntType(ctx.ll_context.i16_type()).into(),
            LirTy::I32 => BasicTypeEnum::IntType(ctx.ll_context.i32_type()).into(),
            LirTy::I64 => BasicTypeEnum::IntType(ctx.ll_context.i64_type()).into(),
            LirTy::I128 => BasicTypeEnum::IntType(ctx.ll_context.i128_type()).into(),
            LirTy::Metadata => BasicMetadataTypeEnum::MetadataType(ctx.ll_context.metadata_type()),
        }
    }

    fn into_basic_type(self, ctx: &CodegenCtx<'ll>) -> BasicTypeEnum<'ll> {
        match self {
            LirTy::I8 => BasicTypeEnum::IntType(ctx.ll_context.i8_type()),
            LirTy::I16 => BasicTypeEnum::IntType(ctx.ll_context.i16_type()),
            LirTy::I32 => BasicTypeEnum::IntType(ctx.ll_context.i32_type()),
            LirTy::I64 => BasicTypeEnum::IntType(ctx.ll_context.i64_type()),
            LirTy::I128 => BasicTypeEnum::IntType(ctx.ll_context.i128_type()),
            LirTy::Metadata => panic!("Metadata type cannot be converted to BasicTypeEnum"),
        }
    }
}
