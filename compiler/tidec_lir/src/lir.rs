use crate::{
    basic_blocks::{BasicBlock, BasicBlockData},
    syntax::{Body, LirTy, Local, LocalData},
};
use tidec_abi::{CodegenBackend, Target, TyAndLayout};
use tidec_utils::index_vec::IdxVec;
use tracing::{debug, instrument};

#[derive(Eq, PartialEq)]
pub struct DefId(usize);

/// Specifies the linkage of a symbol.
/// All Global Variables and Functions have one of the following types of linkage.
///
/// NOTE: A symbol with internal or private linkage must have default visibility.
/// NOTE: It is illegal for a global variable or function declaration to have any linkage type other than external or extern_weak.
pub enum Linkage {
    /// Global values with "private" linkage are only directly accessible by objects in
    /// the current module. In particular, linking code into a module with a private global
    /// value may cause the private to be renamed as necessary to avoid collisions. Because
    /// the symbol is private to the module, all references can be updated. This doesn't show
    /// up in any symbol table in the object file.
    Private,
    /// Similar to private, but the value shows as a local symbol (STB_LOCAL in the case of ELF)
    /// in the object file. This corresponds to the notion of the `static` keyword in C.
    Internal,
    /// Globals with "available_externally" linkage are never emitted into the object file corresponding
    /// to the backend (e.g., LLVM) module. From the linker's perspective, an available_externally global
    /// is equivalent to an external declaration. They exist to allow inlining and other optimizations to
    /// take place given knowledge of the definition of the global, which is known to be somewhere outside
    /// the module. Globals with available_externally linkage are allowed to be discarded at will, and allow
    /// inlining and other optimizations. This linkage type is only allowed on definitions, not declarations.
    AvailableExternally,
    /// Globals with "linkonce" linkage are merged with other globals of the same name when linkage occurs.
    /// This can be used to implement some forms of inline functions, templates, or other code which must be
    /// generated in each translation unit that uses it, but where the body may be overridden with a more
    /// definitive definition later. Unreferenced linkonce globals are allowed to be discarded. Note that
    /// linkonce linkage does not actually allow the optimizer to inline the body of this function into
    /// callers because it doesn't know if this definition of the function is the definitive definition within
    /// the program or whether it will be overridden by a stronger definition. To enable inlining and other
    /// optimizations, use "linkonce_odr" linkage.
    LinkOnce,
    /// "weak" linkage has the same merging semantics as linkonce linkage, except that unreferenced globals with
    /// weak linkage may not be discarded. This is used for globals that are declared "weak" in C source code.
    Weak,
    /// "common" linkage is most similar to "weak" linkage, but they are used for tentative definitions in C,
    /// such as "int X;" at global scope. Symbols with "common" linkage are merged in the same way as weak symbols,
    /// and they may not be deleted if unreferenced. common symbols may not have an explicit section, must have a
    /// zero initializer, and may not be marked 'constant'. Functions and aliases may not have common linkage.
    Common,
    /// "appending" linkage may only be applied to global variables of pointer to array type. When two global
    /// variables with appending linkage are linked together, the two global arrays are appended together.
    /// This is the LLVM, typesafe, equivalent of having the system linker append together "sections" with
    /// identical names when .o files are linked.
    ///
    /// LLVM SPECIFIC: Unfortunately this doesn't correspond to any feature in .o files, so it can only be used for variables
    /// like llvm.global_ctors which llvm interprets specially.
    Appending,
    /// The semantics of this linkage follow the ELF object file model: the symbol is weak until linked,
    /// if not linked, the symbol becomes null instead of being an undefined reference.
    ExternWeak,
    /// The odr suffix indicates that all globals defined with the given name are equivalent, along the lines
    /// of the C++ "one definition rule" ("ODR"). Informally, this means we can inline functions and fold loads
    /// of constants.
    /// Formally, use the following definition: when an odr function is called, one of the definitions is
    /// non-deterministically chosen to run. For odr variables, if any byte in the value is not equal in all
    /// initializers, that byte is a poison value. For aliases and ifuncs, apply the rule for the underlying function or variable.
    ///
    /// These linkage types are otherwise the same as their non-odr versions.
    LinkOnceODR,
    WeakODR,
    /// If none of the above identifiers are used, the global is externally visible, meaning that it participates
    /// in linkage and can be used to resolve external symbol references.
    External,
}

/// Specifies the symbol visibility with regards to dynamic linking.
/// All Global Variables and Functions have one of the following visibility styles.
///
/// A symbol with internal or private linkage must have default visibility.
pub enum Visibility {
    /// On targets that use the ELF object file format, default visibility means that
    /// the declaration is visible to other modules and, in shared libraries, means that
    /// the declared entity may be overridden. On Darwin, default visibility means that the
    /// declaration is visible to other modules. On XCOFF, default visibility means no explicit
    /// visibility bit will be set and whether the symbol is visible (i.e "exported") to other
    /// modules depends primarily on export lists provided to the linker. Default visibility
    /// corresponds to "external linkage" in the language.
    Default,
    /// Two declarations of an object with hidden visibility refer to the same object if they
    /// are in the same shared object. Usually, hidden visibility indicates that the symbol will
    /// not be placed into the dynamic symbol table, so no other module (executable or shared library)
    /// can reference it directly.
    Hidden,
    // On ELF, protected visibility indicates that the symbol will be placed in the dynamic symbol table,
    // but that references within the defining module will bind to the local symbol. That is, the symbol
    // cannot be overridden by another module.
    Protected,
}

pub struct LirBodyMetadata {
    /// The definition ID of the function.
    pub id: DefId,
    /// The name of the function.
    pub name: String,

    /// If the function should be inlined.
    pub inlined: bool,
    /// The linkage of the function.
    pub linkage: Linkage,
    /// The visibility of the function.
    pub visibility: Visibility,
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

#[derive(Debug)]
pub struct LirTyCtx {
    target: Target,
}

impl LirTyCtx {
    #[instrument]
    pub fn new(codegen_backend: CodegenBackend) -> Self {
        let target = Target::new(codegen_backend);
        let ctx = LirTyCtx { target };
        debug!("LirTyCtx created: {:?}", ctx);
        ctx
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy> {
        todo!()
    }
}
