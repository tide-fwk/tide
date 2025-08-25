use crate::{lir::OperandVal, traits::LayoutOf};
use tidec_abi::calling_convention::function::{FnAbi, PassMode};
use tidec_lir::{
    basic_blocks::{BasicBlock, BasicBlockData},
    lir::{LirBody, LirUnit},
    syntax::{LirTy, Local, RETURN_LOCAL, Statement, Terminator},
};
use tidec_utils::index_vec::IdxVec;
use tracing::{debug, instrument};

use crate::{
    lir::{LocalRef, OperandRef},
    traits::{BuilderMethods, DefineCodegenMethods, PreDefineCodegenMethods},
};

pub struct FnCtx<'a, 'be, B: BuilderMethods<'a, 'be>> {
    /// The function ABI.
    /// This contains information about the calling convention,
    /// argument types, return type, etc.
    pub fn_abi: FnAbi<LirTy>,

    /// The body of the function in LIR.
    pub lir_body: &'a LirBody,

    /// The function value.
    /// This is the function that will be generated.
    pub fn_value: B::Value,

    /// The codegen context.
    pub ctx: &'a B::CodegenCtx,

    /// The allocated locals and temporaries for the function.
    ///
    /// Note that the `B::Value` type is used to represent the local references.
    pub locals: IdxVec<Local, LocalRef<B::Value>>,

    /// A cache of the basic blocks in the function.
    /// This is also used to avoid creating multiple basic blocks for the same LIR basic block.
    pub cached_bbs: IdxVec<BasicBlock, Option<B::BasicBlock>>,
}

impl<'ctx, 'll, B: BuilderMethods<'ctx, 'll>> FnCtx<'ctx, 'll, B> {
    /// Codegen the given LIR basic block.
    /// This creates a new builder for the basic block and generates the instructions in it.
    /// It also updates the `cached_bbs` field to avoid creating multiple basic blocks for the same LIR basic block.
    /// Note that this function does not handle unreachable blocks.
    pub fn codegen_basic_block(&mut self, bb: BasicBlock) {
        let be_bb = self.get_or_insert_bb(bb);
        let mut builder = B::build(self.ctx, be_bb);
        let bb_data: &BasicBlockData = &self.lir_body.basic_blocks[bb];
        debug!("Codegen basic block {:?}: {:?}", bb, bb_data);
        for stmt in &bb_data.statements {
            self.codegen_statement(&mut builder, stmt);
        }
        let term = &bb_data.terminator;
        self.codegen_terminator(&mut builder, term);
    }

    /// Get the backend basic block for the given LIR basic block.
    /// If it does not exist, create it and cache it.
    pub fn get_or_insert_bb(&mut self, bb: BasicBlock) -> B::BasicBlock {
        if let Some(Some(be_bb)) = self.cached_bbs.get(bb) {
            return *be_bb;
        }

        let be_bb = B::append_basic_block(&self.ctx, self.fn_value, "");
        self.cached_bbs[bb] = Some(be_bb);
        be_bb
    }

    /// Codegen the given LIR statement.
    /// This function is called by `codegen_basic_block` for each statement in the basic block.
    /// It generates the corresponding instructions in the backend.
    fn codegen_statement(&mut self, builder: &mut B, stmt: &Statement) {
        let _ = (builder, stmt);
        todo!("Implement codegen_statement");
    }

    /// Codegen the given LIR terminator.
    /// This function is called by `codegen_basic_block` for the terminator of the basic block.
    /// It generates the corresponding instructions in the backend.
    fn codegen_terminator(&mut self, builder: &mut B, term: &Terminator) {
        debug!("Codegen terminator: {:?}", term);
        match term {
            Terminator::Return => self.codegen_return_terminator(builder),
        }
    }

    /// Codegen a return terminator.
    /// This function generates the return instruction for the function.
    /// It handles different return modes based on the function ABI.
    fn codegen_return_terminator(&mut self, builder: &mut B) {
        let be_val = match self.fn_abi.ret.mode {
            PassMode::Ignore | PassMode::Indirect => {
                builder.build_return(None);
                return;
            }
            PassMode::Direct => {
                let operand_ref = self.codegen_operand(builder, RETURN_LOCAL);
                match operand_ref.operand_val {
                    OperandVal::Ref(_) => todo!("Handle return by reference — load from place"),
                    _ => todo!()
                }

            }
        };

        todo!()
    }

    fn codegen_operand(&mut self, builder: &mut B, local: Local) -> OperandRef<B::Value> {
        let layout = builder
            .ctx()
            .layout_of(self.lir_body.ret_and_args[local].ty);

        if layout.is_zst() {
            return OperandRef::new_zst(layout);
        }

        let local_ref = &self.locals[local];
        match local_ref {
            LocalRef::OperandRef(operand_ref) => *operand_ref,
            LocalRef::PlaceRef(place_ref) => {
                builder.load_operand(place_ref)
            }
        }
    }
}

#[instrument(skip(ctx, lir_unit))]
pub fn compile_lir_unit<'a, 'be, B: BuilderMethods<'a, 'be>>(
    ctx: &'a B::CodegenCtx,
    lir_unit: LirUnit,
) {
    // Predefine the functions. That is, create the function declarations.
    for lir_body in &lir_unit.bodies {
        ctx.predefine_body(&lir_body.metadata, &lir_body.ret_and_args);
    }

    // Now that all functions are pre-defined, we can compile the bodies.
    for lir_body in &lir_unit.bodies {
        // It corresponds to:
        // ```rust
        // for &(mono_item, item_data) in &mono_items {
        //     mono_item.define::<Builder<'_, '_, '_>>(&mut cx, cgu_name.as_str(), item_data);
        // }
        // ```
        // in rustc_codegen_llvm/src/base.rs
        // lir::define_lir_body::<B>(ctx, lir_body);
        ctx.define_body(lir_body);
    }
}
