use tidec_abi::layout::TyAndLayout;
use crate::lir::LirTyCtx;

pub struct LayoutCtx<'a> {
    lir_ty_ctx: &'a LirTyCtx,
}

impl<'a> LayoutCtx<'a> {
    pub fn new(lir_ty_ctx: &'a LirTyCtx) -> Self {
        LayoutCtx { lir_ty_ctx }
    }

    /// Computes the layout for a given type. We should cache the results
    /// to avoid recomputing the layout for the same type multiple times.
    pub fn compute_layout<T>(&self, ty: T) -> TyAndLayout<T> {
        let _ = ty;
        // let data_layout = self.target.data_layout;
        unimplemented!()
    }
}
