use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

pub struct CodeGenCtx<'ll> {
    pub ll_context: &'ll Context,
    pub ll_module: &'ll Module<'ll>,
}

impl CodeGenCtx<'_> {
    pub fn new<'ll>(module_name: &str) -> Self {
        unsafe {
            let context = &mut Context::create() as *mut Context;
            let module = &(*context).create_module(module_name) as *const Module;
            CodeGenCtx {
                ll_context: &*context,
                ll_module: &*module,
            }
        }
    }
}

pub struct CodeGenBuilder<'ll> {
    pub builder: &'ll mut Builder<'ll>,
    pub ctx: CodeGenCtx<'ll>,
}

impl CodeGenBuilder<'_> {
    pub fn new<'ll>(ctx: CodeGenCtx<'ll>) -> CodeGenBuilder<'ll> {
        unsafe {
            let builder = &mut (*ctx.ll_context).create_builder() as *mut Builder;
            CodeGenBuilder {
                builder: &mut *builder,
                ctx,
            }
        }
    }
}
