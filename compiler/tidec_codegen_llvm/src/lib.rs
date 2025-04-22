pub mod lir;

use std::ops::Deref;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::FunctionValue;
use inkwell::{basic_block::BasicBlock, builder::Builder};

use lir::types::{IntoBasicType, IntoBasicTypeMetadata};
use tidec_lir::lir::{LirBody, Metadata};
use tidec_lir::syntax::RETURN_PLACE;

pub trait CodeGen<'ll> {
    fn new(ll_context: &'ll Context, ll_module: Module<'ll>) -> Self;
    fn get_fn(&self, name: &str) -> Option<FunctionValue<'ll>>;
    fn new_fn(&self, lir_body: &LirBody) -> FunctionValue<'ll>;
}

pub struct CodeGenCtx<'ll> {
    // FIXME: Make this private
    pub ll_context: &'ll Context,
    pub ll_module: Module<'ll>,
}

impl<'ll> CodeGenCtx<'ll> {
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

impl<'ll> CodeGen<'ll> for CodeGenCtx<'ll> {
    fn new(ll_context: &'ll Context, ll_module: Module<'ll>) -> CodeGenCtx<'ll> {
        CodeGenCtx {
            ll_context,
            ll_module,
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

        let ret_ty = lir_body.ret_args[RETURN_PLACE].ty.into_basic_type(&self);
        let formal_param_tys = lir_body.ret_args.as_slice()[RETURN_PLACE..]
            .iter()
            .map(|local_data| local_data.ty.into_basic_type_metadata(&self))
            .collect::<Vec<_>>();

        let fn_ty = self.declare_fn(ret_ty, formal_param_tys.as_slice());
        let fn_val = self.ll_module.add_function(name, fn_ty, None);

        fn_val
    }
}

impl Deref for CodeGenCtx<'_> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ll_context
    }
}

pub struct CodeGenBuilder<'ll> {
    pub builder: Builder<'ll>,
    pub ctx: CodeGenCtx<'ll>,
}

impl<'ll> From<CodeGenCtx<'ll>> for CodeGenBuilder<'ll> {
    fn from(ctx: CodeGenCtx<'ll>) -> Self {
        CodeGenBuilder {
            builder: ctx.ll_context.create_builder(),
            ctx,
        }
    }
}

// =================

pub trait BuilderMethods<'ll> {
    type CodeGenCtx: CodeGen<'ll>;

    fn build(ctx: Self::CodeGenCtx, llbb: BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &Self::CodeGenCtx,
        fn_value: FunctionValue<'ll>,
        name: &str,
    ) -> BasicBlock<'ll>;
}

impl<'ll> BuilderMethods<'ll> for CodeGenBuilder<'ll> {
    type CodeGenCtx = CodeGenCtx<'ll>;

    /// Create a new CodeGenBuilder from a CodeGenCtx and a BasicBlock.
    /// The builder is positioned at the end of the BasicBlock.
    fn build(ctx: Self::CodeGenCtx, llbb: BasicBlock) -> Self {
        let builder = CodeGenBuilder::from(ctx);
        builder.builder.position_at_end(llbb);
        builder
    }

    /// Append a new basic block to the function.
    fn append_basic_block(
        ctx: &Self::CodeGenCtx,
        fn_value: FunctionValue<'ll>,
        name: &str,
    ) -> BasicBlock<'ll> {
        ctx.ll_context.append_basic_block(fn_value, name)
    }
}

struct FnCtx {
    // pub locals: IdxVec<Local, LocalRef>,
}

fn compile_lir_body<'ll, B: BuilderMethods<'ll>>(ctx: B::CodeGenCtx, lir_body: LirBody) {
    let fn_value = ctx.new_fn(&lir_body);
    let entry_bb = B::append_basic_block(&ctx, fn_value, "entry");
    let builder = B::build(ctx, entry_bb);

    // Initialize the locals

    // Compile the basic blocks
    for bb in lir_body.basic_blocks.iter() {
        // let llbb = LLVMBasicBlock::from(bb);
        // let builder = CodeGenBuilder::new(ctx);
    }
}

fn compile_codegen_unit<'ll, B: BuilderMethods<'ll>>(metadata: Metadata, lir_body: LirBody) {
    let ll_context = Context::create();
    let ll_module = ll_context.create_module(&metadata.module_name);
    let ctx = CodeGenCtx::new(&ll_context, ll_module);

    // FIXME: for each body we need to create a function
    compile_lir_body::<CodeGenBuilder>(ctx, lir_body);
}
