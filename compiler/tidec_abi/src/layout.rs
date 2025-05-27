use crate::size_and_align::{AbiAndPrefAlign, Size};

/// Represents a type along with its size and alignment information.
///
/// This is commonly used during codegen and layout computation to reason about
/// how values should be represented in memory on the target platform.
pub struct TyAndLayout<T> {
    /// The type this layout refers to.
    ///
    /// This is usually a LIR type, but can be any type that has a size and alignment.
    pub ty: T,
    /// The layout information for the type, including size and alignment.
    pub layout: Layout,
}

/// Represents the layout of a type in the target architecture.
///
/// This struct contains the size, alignment, and backend representation
/// of a type, which is essential for code generation and memory layout decisions.
pub struct Layout {
    /// The size of the type in bytes.
    pub size: Size,
    /// The ABI and preferred alignment of the type.
    pub align: AbiAndPrefAlign,
    /// The backend representation of the type, which may include additional
    pub backend_repr: BackendRepr,
}

pub enum BackendRepr {
    Todo,
}

pub struct LayoutCtx {}


impl LayoutCtx {
    pub fn new() -> Self {
        LayoutCtx {}
    }

    /// Computes the layout for a given type.
    pub fn compute_layout<T>(&self, ty: T) -> TyAndLayout<T> {

    }
}
