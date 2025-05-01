use std::ops::Deref;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{TargetData, TargetTriple};
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::FunctionValue;
use tracing::instrument;

use crate::lir::types::BasicTypesUtils;
use crate::CodegenMethods;
use tidec_lir::lir::{LirBody, LirTyCtx};
use tidec_lir::syntax::RETURN_PLACE;

pub struct CodegenCtx<'ll> {
    // FIXME: Make this private
    pub ll_context: &'ll Context,
    pub ll_module: Module<'ll>,

    pub lir_ty_ctx: LirTyCtx,
    // TODO: Add filelds from rustc/compiler/rustc_codegen_llvm/src/context.rs
}

impl<'ll> Deref for CodegenCtx<'ll> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ll_context
    }
}

impl<'ll> CodegenCtx<'ll> {
    fn declare_fn(
        &self,
        ret_ty: BasicTypeEnum<'ll>,
        param_tys: &[BasicMetadataTypeEnum<'ll>],
    ) -> FunctionType<'ll> {
        let fn_ty = match ret_ty {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(param_tys, false),
            BasicTypeEnum::ArrayType(array_type) => array_type.fn_type(param_tys, false),
            BasicTypeEnum::FloatType(float_type) => float_type.fn_type(param_tys, false),
            BasicTypeEnum::PointerType(pointer_type) => pointer_type.fn_type(param_tys, false),
            BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(param_tys, false),
            BasicTypeEnum::VectorType(vector_type) => vector_type.fn_type(param_tys, false),
        };

        fn_ty
    }
}

impl<'ll> CodegenMethods<'ll> for CodegenCtx<'ll> {
    #[instrument(skip(lir_ty_ctx, ll_context, ll_module))]
    fn new(
        lir_ty_ctx: LirTyCtx,
        ll_context: &'ll Context,
        ll_module: Module<'ll>,
    ) -> CodegenCtx<'ll> {
        let target = lir_ty_ctx.target();
        let data_layout_string = target.data_layout_string();
        let target_triple_string = target.target_triple_string();

        ll_module.set_triple(&TargetTriple::create(&target_triple_string));
        // TODO: As TargetData contains methods to know the size, align, etc... for each LLVM type
        // we could consider to store it in a context
        ll_module.set_data_layout(&TargetData::create(&data_layout_string).get_data_layout());

        CodegenCtx {
            ll_context,
            ll_module,
            lir_ty_ctx,
        }
    }

    fn get_fn(&self, name: &str) -> Option<FunctionValue<'ll>> {
        self.ll_module.get_function(name)
    }

    fn new_fn(&self, lir_body: &LirBody) -> FunctionValue<'ll> {
        let name = lir_body.metadata.name.as_str();

        if let Some(f) = self.get_fn(name) {
            return f;
        }

        let ret_ty = lir_body.ret_and_args[RETURN_PLACE]
            .ty
            .into_basic_type(&self);
        let formal_param_tys = lir_body.ret_and_args.as_slice()[RETURN_PLACE..]
            .iter()
            .map(|local_data| local_data.ty.into_basic_type_metadata(&self))
            .collect::<Vec<_>>();

        let fn_ty = self.declare_fn(ret_ty, formal_param_tys.as_slice());
        let fn_val = self.ll_module.add_function(name, fn_ty, None);

        fn_val
    }
}
