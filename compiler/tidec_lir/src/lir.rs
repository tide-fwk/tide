use crate::{
    basic_blocks::{BasicBlock, BasicBlockData},
    syntax::{Body, LirTy, Local, LocalData},
};
use tidec_abi::{CodegenBackend, TargetDataLayout, TyAndLayout};
use tidec_utils::index_vec::IdxVec;

#[derive(Eq, PartialEq)]
pub struct DefId(usize);

pub struct LirBodyMetadata {
    /// The name of the function.
    pub name: String,

    /// The definition ID of the function.
    pub id: DefId,
}

/// The body of a function in LIR (Low-level Intermediate Representation).
pub struct LirBody {
    /// The metadata of the function.
    pub metadata: LirBodyMetadata,

    /// The locals for return value and arguments of the function.
    pub ret_and_args: IdxVec<Local, LocalData>,

    /// The rest of the locals.
    pub locals: IdxVec<Local, LocalData>,

    /// The basic blocks of the function.
    pub basic_blocks: IdxVec<BasicBlock, BasicBlockData>,
}

/// The metadata of a LIR unit (module).
pub struct LirUnitMetadata {
    pub unit_name: String,
}

/// The LIR unit (module).
pub struct LirUnit {
    /// The metadata of the unit.
    pub metadata: LirUnitMetadata,

    /// The functions in the unit.
    pub body: IdxVec<Body, LirBody>,
}

pub struct LirTyCtx {
    codegen_backend: CodegenBackend,

    /// The target data layout.
    target_data_layout: TargetDataLayout,
}

impl LirTyCtx {
    /// Create a new LIR type context.
    pub fn new(codegen_backend: CodegenBackend, target_data_layout: TargetDataLayout) -> Self {
        LirTyCtx {
            codegen_backend,
            target_data_layout,
        }
    }

    pub fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy> {
        todo!()
    }
}
