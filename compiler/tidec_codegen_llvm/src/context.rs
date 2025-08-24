use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{TargetData, TargetTriple};
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum};
use tidec_codegen_ssa::lir;
use tidec_utils::index_vec::IdxVec;
use tracing::{debug, instrument};

use crate::lir::lir_body_metadata::{
    CallConvUtils, LinkageUtils, UnnamedAddressUtils, VisibilityUtils,
};
use crate::lir::lir_ty::BasicTypesUtils;
use tidec_codegen_ssa::traits::{
    CodegenBackend, CodegenBackendTypes, CodegenMethods, DefineCodegenMethods,
    PreDefineCodegenMethods,
};
use tidec_lir::lir::{DefId, LirBody, LirBodyMetadata, LirTyCtx};
use tidec_lir::syntax::{Local, LocalData, RETURN_LOCAL};

// TODO: Add filelds from rustc/compiler/rustc_codegen_llvm/src/context.rs
pub struct CodegenCtx<'ll> {
    // FIXME: Make this private
    pub ll_context: &'ll Context,
    // FIXME: Make this private
    pub ll_module: Module<'ll>,

    /// The LIR type context.
    pub lir_ty_ctx: LirTyCtx,

    /// A map from DefId to the LLVM value (usually a function value).
    //
    // FIXME: Consider removing RefCell and using &mut
    //
    // TODO: Probably we could remove this and use only the module to find functions (more efficient?).
    // Something like: `self.ll_module.get_function(<name>)` (see `get_fn`).
    pub instances: RefCell<HashMap<DefId, AnyValueEnum<'ll>>>,
}

impl<'ll> Deref for CodegenCtx<'ll> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ll_context
    }
}

impl<'ll> CodegenBackendTypes for CodegenCtx<'ll> {
    type BasicBlock = BasicBlock<'ll>;
    type FunctionType = FunctionType<'ll>;
    type Type = BasicTypeEnum<'ll>;
    type Value = AnyValueEnum<'ll>;
    type MetadataType = BasicMetadataTypeEnum<'ll>;
    type MetadataValue = BasicMetadataValueEnum<'ll>;
}

impl<'ll> CodegenBackend for CodegenCtx<'ll> {
    type Context = Context;
    type Module = Module<'ll>;
}

impl PreDefineCodegenMethods for CodegenCtx<'_> {
    fn predefine_body(
        &self,
        lir_body_metadata: &LirBodyMetadata,
        lir_body_ret_and_args: &IdxVec<Local, LocalData>,
    ) {
        let name = lir_body_metadata.name.as_str();

        let ret_ty = lir_body_ret_and_args[RETURN_LOCAL].ty.into_basic_type(self);
        let formal_param_tys = lir_body_ret_and_args.as_slice()[RETURN_LOCAL..]
            .iter()
            .map(|local_data| local_data.ty.into_basic_type_metadata(self))
            .collect::<Vec<_>>();
        let fn_ty = self.declare_fn(ret_ty, formal_param_tys.as_slice());
        let linkage = lir_body_metadata.linkage.into_linkage();
        let calling_convention = lir_body_metadata.call_conv.into_call_conv();
        let fn_val = self.ll_module.add_function(name, fn_ty, Some(linkage));
        fn_val.set_call_conventions(calling_convention);

        let fn_global_value = fn_val.as_global_value();
        let visibility = lir_body_metadata.visibility.into_visibility();
        fn_global_value.set_visibility(visibility);
        let unnamed_addr = lir_body_metadata.unnamed_address.into_unnamed_address();
        fn_global_value.set_unnamed_address(unnamed_addr);

        debug!(
            "get_or_declare_fn((name: {}, ret_ty: {:?}, param_tys: {:?}, linkage: {:?}, visibility: {:?}, calling_convention: {:?}, unnamed_addr: {:?})) delared",
            name, ret_ty, formal_param_tys, linkage, visibility, calling_convention, unnamed_addr
        );

        self.instances.borrow_mut().insert(
            lir_body_metadata.def_id,
            AnyValueEnum::FunctionValue(fn_val),
        );
    }
}

impl DefineCodegenMethods for CodegenCtx<'_> {
    /// For LLVM, we are able to reuse the generic implementation of `define_lir_body`
    /// provided in the `lir` module, as it is generic over the `BuilderMethods` trait.
    fn define_body(&self, lir_body: &LirBody) {
        lir::codegen_lir_body::<'_, '_, crate::builder::CodegenBuilder<'_, '_>>(self, lir_body);
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
            instances: RefCell::new(HashMap::new()),
        }
    }

    fn get_fn(&self, lir_body_metadata: &LirBodyMetadata) -> Option<AnyValueEnum<'ll>> {
        let name = lir_body_metadata.name.as_str();

        if let Some(instance) = self.instances.borrow().get(&lir_body_metadata.def_id) {
            debug!("get_fn(name: {}) found in instances", name);
            return Some(instance.clone());
        }

        if let Some(f) = self.ll_module.get_function(name) {
            debug!("get_fn(name: {}) found in module", name);
            return Some(AnyValueEnum::FunctionValue(f));
        }

        debug!("get_fn(name: {}) not found", name);
        None
    }

    /// TODO(bruzzone): We expect this function returns a function value.
    fn get_or_define_fn(
        &self,
        lir_body_metadata: &LirBodyMetadata,
        lir_body_ret_and_args: &IdxVec<Local, LocalData>,
    ) -> AnyValueEnum<'ll> {
        let name = lir_body_metadata.name.as_str();

        if let Some(fn_val) = self.get_fn(lir_body_metadata) {
            debug!("get_or_define_fn(name: {}) found", name);
            return fn_val;
        }

        // TODO: fallback by declaring the function
        self.predefine_body(lir_body_metadata, lir_body_ret_and_args);
        let fn_val = self
            .get_fn(lir_body_metadata)
            .expect("function should be defined after predefine_body");

        AnyValueEnum::FunctionValue(fn_val.into_function_value())
    }
}
