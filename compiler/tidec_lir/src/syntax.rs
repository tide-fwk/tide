use tidec_utils::idx::Idx;

#[derive(Copy, Clone)]
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
pub struct Local(usize);

#[derive(Eq, PartialEq)]
pub struct Body(usize);

pub const RETURN_PLACE: Local = Local(0);

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
    Assign(Local, RValue),
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
