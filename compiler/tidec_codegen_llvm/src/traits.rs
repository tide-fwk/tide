use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use tidec_abi::TyAndLayout;
use tidec_lir::lir::{LirBody, LirTyCtx};
use tidec_lir::syntax::LirTy;

// TODO: This trait should be generic over the LLVM backend.
pub trait CodegenMethods<'ll> {
    fn new(lir_ty_ctx: LirTyCtx, ll_context: &'ll Context, ll_module: Module<'ll>) -> Self;
    fn get_fn(&self, name: &str) -> Option<FunctionValue<'ll>>;
    fn new_fn(&self, lir_body: &LirBody) -> FunctionValue<'ll>;
}

// =================

// TODO: Make CodegenMethods generic (not LLVM specific)
pub trait BuilderMethods<'a, 'll> {
    type CodegenCtx: CodegenMethods<'ll>;

    fn build(ctx: &'a Self::CodegenCtx, llbb: BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &Self::CodegenCtx,
        fn_value: FunctionValue<'ll>, // TODO: Make FunctionValue generic
        name: &str,
    ) -> BasicBlock<'ll>;

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy>;
}
