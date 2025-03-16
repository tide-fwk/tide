use crate::lir::syntax::{Statement, Terminator};
use tidec_utils::{idx::Idx, index_vec::IdxVec};

#[derive(PartialEq, Eq)]
struct BasicBlock(usize);

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

pub struct BasicBlockData {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub struct BasicBlocks {
    basic_blocks: IdxVec<BasicBlock, BasicBlockData>,
}
