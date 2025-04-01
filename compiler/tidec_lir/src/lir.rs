use tidec_utils::index_vec::IdxVec;
use crate::{basic_blocks::{BasicBlock, BasicBlockData}, syntax::{Local, LocalData}};

#[derive(Eq, PartialEq)]
pub struct DefId(usize);

/// The metadata of a module.
pub struct Metadata {
    pub module_name: String,
}

pub struct LirMetadata {
    /// The name of the function.
    pub name: String,

    /// The definition ID of the function.
    pub id: DefId,

}

/// The body of a function in LIR (Low-level Intermediate Representation).
pub struct LirBody {
    /// The metadata of the function.
    pub metadata: LirMetadata,

    /// The locals for return value and arguments of the function.
    pub ret_args: IdxVec<Local, LocalData>,

    /// The rest of the locals.
    pub locals: IdxVec<Local, LocalData>,

    /// The basic blocks of the function.
    pub basic_blocks: IdxVec<BasicBlock, BasicBlockData>,
}

