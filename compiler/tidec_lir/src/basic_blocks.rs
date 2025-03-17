use tidec_utils::{idx::Idx, index_vec::IdxVec};

use crate::syntax::{Local, LocalData, Statement, Terminator};

#[derive(Eq, PartialEq)]
struct BasicBlock(usize);

/// The data of a basic block.
///
/// A basic block is a sequence of statements that ends with a terminator.
/// The terminator is the last statement of the block and transfers control to another block.
struct BasicBlockData {
    statements: Vec<Statement>,
    terminator: Terminator,
}

/// The body of a function in LIR (Low-level Intermediate Representation).
pub struct LirBody {
    /// The locals for return value and arguments of the function.
    pub ret_args: IdxVec<Local, LocalData>,

    /// The rest of the locals.
    pub locals: IdxVec<Local, LocalData>,

    /// The basic blocks of the function.
    pub basic_blocks: IdxVec<BasicBlock, BasicBlockData>,
}

////////// Trait implementations  //////////

impl Idx for BasicBlock {
    fn new(idx: usize) -> Self {
        BasicBlock(idx)
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
