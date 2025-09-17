use crate::lir::LirCtx;
use crate::syntax::LirTy;
use tidec_abi::{
    layout::{BackendRepr, Layout, Primitive, TyAndLayout},
    size_and_align::{AbiAndPrefAlign, Size},
};

pub struct LayoutCtx<'a> {
    lir_ty_ctx: &'a LirCtx,
}

impl<'a> LayoutCtx<'a> {
    // It accepts the `LirTyCtx` because it contains the `TargetDataLayout`.
    pub fn new(lir_ty_ctx: &'a LirCtx) -> Self {
        LayoutCtx { lir_ty_ctx }
    }

    /// Computes the layout for a given type. We should cache the results
    /// to avoid recomputing the layout for the same type multiple times.
    pub fn compute_layout(&self, ty: LirTy) -> TyAndLayout<LirTy> {
        let data_layout = &self.lir_ty_ctx.target().data_layout;

        let (size, align, backend_repr) = match ty {
            LirTy::I8 => (
                Size::from_bits(8),
                data_layout.i8_align,
                BackendRepr::Scalar(Primitive::I8),
            ),
            LirTy::I16 => (
                Size::from_bits(16),
                data_layout.i16_align,
                BackendRepr::Scalar(Primitive::I16),
            ),
            LirTy::I32 => (
                Size::from_bits(32),
                data_layout.i32_align,
                BackendRepr::Scalar(Primitive::I32),
            ),
            LirTy::I64 => (
                Size::from_bits(64),
                data_layout.i64_align,
                BackendRepr::Scalar(Primitive::I64),
            ),
            LirTy::I128 => (
                Size::from_bits(128),
                data_layout.i128_align,
                BackendRepr::Scalar(Primitive::I128),
            ),
            LirTy::Metadata => (
                Size::from_bits(0),
                AbiAndPrefAlign::new(1, 1),
                BackendRepr::Memory,
            ),
        };

        TyAndLayout {
            ty,
            layout: Layout {
                size,
                align,
                backend_repr,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lir::{EmitKind, LirCtx};
    use tidec_abi::target::BackendKind;

    #[test]
    fn test_layout_ctx_new() {
        let lir_ctx = LirCtx::new(BackendKind::Llvm, EmitKind::Object);
        let layout_ctx = LayoutCtx::new(&lir_ctx);
        // Test that the context is stored correctly (by reference)
        assert!(std::ptr::eq(layout_ctx.lir_ty_ctx, &lir_ctx));
    }
}
