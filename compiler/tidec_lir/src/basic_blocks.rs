use tidec_utils::{idx::Idx, index_vec::IdxVec};

use crate::syntax::{Local, LocalData, Statement, Terminator};

#[derive(Eq, PartialEq)]
pub struct BasicBlock(usize);

/// The data of a basic block.
///
/// A basic block is a sequence of statements that ends with a terminator.
/// The terminator is the last statement of the block and transfers control to another block.
pub struct BasicBlockData {
    statements: Vec<Statement>,
    terminator: Terminator,
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
