use crate::lir::LirCtx;
use tidec_abi::{
    layout::{BackendRepr, Layout, Primitive, TyAndLayout},
    size_and_align::{AbiAndPrefAlign, Size},
};

pub struct LayoutCtx<'a> {
    _lir_ty_ctx: &'a LirCtx,
}

impl<'a> LayoutCtx<'a> {
    // It accepts the `LirTyCtx` because it contains the `TargetDataLayout`.
    pub fn new(lir_ty_ctx: &'a LirCtx) -> Self {
        LayoutCtx {
            _lir_ty_ctx: lir_ty_ctx,
        }
    }

    /// Computes the layout for a given type. We should cache the results
    /// to avoid recomputing the layout for the same type multiple times.
    pub fn compute_layout<T>(&self, ty: T) -> TyAndLayout<T> {
        let _ = ty;
        // let data_layout = self.target.data_layout;

        // HARDCODE FOR TESTING an integer type
        TyAndLayout {
            ty,
            layout: Layout {
                size: Size::from_bits(32),
                align: AbiAndPrefAlign::new(4, 4),
                backend_repr: BackendRepr::Scalar(Primitive::I32),
            },
        }
    }
}
