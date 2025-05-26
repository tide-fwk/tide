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

pub(crate) enum RValue {
    // TODO: Implement
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
pub(crate) enum Statement {
    Assign(Box<(Place, RValue)>),
}

/// The terminator of a basic block.
///
/// The terminator of a basic block is the last statement of the block.
/// It is an operation that ends the block and transfers control to another block.
pub(crate) enum Terminator {
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
