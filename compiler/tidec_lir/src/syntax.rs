use std::num::NonZero;

use tidec_utils::idx::Idx;

#[derive(Debug, Copy, Clone)]
pub enum LirTy {
    I8,
    I16,
    I32,
    I64,
    I128,

    // https://llvm.org/docs/TypeMetadata.html
    Metadata,
}

#[derive(Eq, PartialEq)]
/// A `Local` variable in the LIR.
///
/// `Local` acts as an index into the set of local variables declared within a function or
/// basic block. These variables include user-declared bindings, temporaries created
/// during compilation, and compiler-generated variables such as those for intermediate
/// values or storage management.
///
/// The index (`usize`) identifies the local variable uniquely within its context.
/// The zeroth local (`Local(0)`) often refers to the return place of a function.
pub struct Local(usize);
pub const RETURN_LOCAL: Local = Local(0);

/// Represents a memory location (or "place") within LIR that can be used
/// as the target of assignments or the source of loads.
///
/// A `Place` consists of:
/// - A `local`: the base variable or temporary (identified by a `Local`)
/// - A `projection`: a sequence of projections used to navigate through the
///   structure of compound types (e.g., fields, dereferences, array indexing).
///
/// For example, in the Rust expression `x.0.y`, the base local would be `x`, and the
/// projection would include field accessors to reach `.0` and `.y`.
///
/// `Place`s are used in LIR to abstract over memory references in a type-safe and
/// structured manner, allowing the compiler to track aliasing, lifetimes,
/// and optimize memory access.
pub struct Place {
    /// The base local variable from which this place starts.
    pub local: Local,

    /// A (possibly empty) list of projections representing access to subparts
    /// of the base local, such as fields or dereferenced pointers.
    pub projection: Vec<Projection>,
}

/// Represents a single step in a `Place` projection path.
///
/// A `Projection` allows navigation into more complex data structures
/// from a base `Local`. Multiple projections can be chained to model
/// deeply nested memory accesses.
///
/// Common projection types include:
/// - Field access (e.g., `.field`)
/// - Dereferencing a pointer (e.g., `*p`)
/// - Indexing into an array or slice (e.g., `[i]`)
///
/// TODO: This enum is currently a placeholder and should be extended with
/// specific variants such as `Field`, `Deref`, `Index`, etc.
pub enum Projection {
    Todo,
}

#[derive(Eq, PartialEq)]
pub struct Body(usize);

pub enum RValue {
    /// A constant value.
    /// TODO(bruzzone): This could be moved into a separate variant type, i.e., enum Operand { Const(..), Copy(..), Move(..) }
    Const(ConstOperand),
}

#[derive(Debug)]
// TODO(bruzzone): Add more variants for different constant types.
pub enum ConstOperand {
    /// A constant value that can be used in the LIR.
    Value(ConstValue, LirTy),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Represents a constant value.
// TODO(bruzzone): Add slice variant for strings, arrays, etc.
// TODO(bruzzone): Add indirect variant. A value not representable by the other variants; needs to be stored in-memory.
pub enum ConstValue {
    /// A constant value that is a zero-sized type (ZST).
    ZST,
    /// A constant scalar value.
    /// The consts with this variant have typically a layout that is compatible with scalar types, such as integers, floats, or pointers. That is, the backend representation of the constant is a scalar value.
    Scalar(ConstScalar),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Represents a constant scalar value.
// TODO(bruzzone): Add pointer variant for constants that are pointers to other constants or memory locations.
pub enum ConstScalar {
    /// Raw byte representation of the constant.
    Value(RawScalarValue),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// The raw bytes of a simple value.
///
/// This is a packed struct in order to allow this type to be optimally embedded in enums
/// (like Scalar). That is, the size of this type is 17 bytes, and the alignment is 1 byte.
#[repr(C, packed)]
pub struct RawScalarValue {
    /// The first `size` bytes of `data` are the value.
    /// Do not try to read less or more bytes than that, this is UB.
    /// The remaining bytes must be 0.
    pub data: u128,
    pub size: NonZero<u8>,
}

#[derive(Copy, Clone)]
pub struct LocalData {
    pub ty: LirTy,
    pub mutable: bool,
}

/// A statement in a basic block.
///
/// A statement is an operation that does not transfer control to another block.
/// It is a part of the block's execution.
pub enum Statement {
    Assign(Box<(Place, RValue)>),
}

/// The terminator of a basic block.
///
/// The terminator of a basic block is the last statement of the block.
/// It is an operation that ends the block and transfers control to another block.
pub enum Terminator {
    /// Returns from the function.
    ///
    /// The semantics of return is, at least, assign the value in the current
    /// return place (`Local(0)`) to the place specified, via a `Call` terminator
    /// by the caller.
    Return,
}

////////// Trait implementations  //////////

impl Idx for Local {
    fn new(idx: usize) -> Self {
        Local(idx)
    }

    fn idx(&self) -> usize {
        self.0
    }

    fn incr(&mut self) {
        self.0 += 1;
    }

    fn incr_by(&mut self, by: usize) {
        self.0 += by;
    }
}

impl Idx for Body {
    fn new(idx: usize) -> Self {
        Body(idx)
    }

    fn idx(&self) -> usize {
        self.0
    }

    fn incr(&mut self) {
        self.0 += 1;
    }

    fn incr_by(&mut self, by: usize) {
        self.0 += by;
    }
}
