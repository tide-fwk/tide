pub mod calling_convention;

use tracing::{info, instrument};

#[derive(Debug)]
/// The backend kind for code generation.
///
/// This enum represents the different backends that can be used for code generation.
pub enum BackendKind {
    /// The LLVM backend.
    Llvm,

    /// The Cranelift backend.
    Cranelift,

    /// The GCC (GNU Compiler Collection) backend.
    Gcc,
}

// TODO: Other address spaces.
#[derive(Debug)]
pub enum AddressSpace {
    /// The default address space.
    DATA = 0,
}

impl From<&AddressSpace> for u32 {
    fn from(addr_space: &AddressSpace) -> Self {
        match *addr_space {
            AddressSpace::DATA => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Size of a type in bytes.
pub struct Size(u64);

impl Size {
    /// Rounds `bits` up to the next-higher byte boundary, if `bits` is
    /// not a multiple of 8.
    pub fn from_bits(bits: impl TryInto<u64>) -> Size {
        let bits = bits.try_into().ok().unwrap();
        // Avoid potential overflow from `bits + 7`.
        Size(bits / 8 + ((bits % 8) + 7) / 8)
    }

    /// Returns the size in bytes.
    pub fn bytes(&self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub enum AlignError {
    TooLarge(u64),
    NotPowerOfTwo(u64),
}

#[derive(Debug, Clone, Copy)]
/// Alignment of a type in bytes (always a power of two).
pub struct Align(u64);

impl Align {
    #[inline]
    pub fn from_bits(bits: u64) -> Result<Align, AlignError> {
        Align::from_bytes(Size::from_bits(bits).bytes())
    }

    #[inline]
    /// Creates an `Align` from a byte count.
    pub const fn from_bytes(align: u64) -> Result<Align, AlignError> {
        // To prevent overflow.
        // For example, when `align` is 0, `align.trailing_zeros()` is 64.
        // This means that `1 << tz` results in a panic with "attempt to shift left with overflow"
        // because `1` followed by 64 zeros is too large for a u64.
        if align == 0 {
            return Ok(Align(0));
        }

        #[cold]
        const fn not_power_of_2(align: u64) -> AlignError {
            AlignError::NotPowerOfTwo(align)
        }

        #[cold]
        const fn too_large(align: u64) -> AlignError {
            AlignError::TooLarge(align)
        }

        let tz = align.trailing_zeros();
        if align != (1 << tz) {
            return Err(not_power_of_2(align));
        }

        if align > u64::MAX / 8 {
            return Err(too_large(align));
        }

        Ok(Align(align))
    }

    #[inline]
    pub const fn bytes(&self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
/// Specifies both the ABI-required and preferred alignment for a type, in bytes.
///
/// Both `abi` and `pref` are powers of two. The ABI alignment (`abi`) is the minimum
/// required alignment for correct program execution, as defined by the platform's ABI.
/// The preferred alignment (`pref`) is a potentially larger value that may yield better
/// performance on some architectures.
///
/// For example, in LLVM, if a preferred alignment is not explicitly set, it defaults to
/// the ABI alignment.
///
/// This type is commonly used during layout computation and codegen to determine
/// how types should be aligned in memory.
pub struct AbiAndPrefAlign {
    /// The alignment required by the ABI for this type.
    pub abi: Align,
    /// The preferred alignment for this type, which may be larger than the ABI alignment.
    pub pref: Align,
}

impl AbiAndPrefAlign {
    /// Creates a new `AbiAndPrefAlign` with the specified ABI and preferred
    /// alignment in bytes.
    pub fn new(abi: u64, pref: u64) -> Self {
        Self {
            abi: Align::from_bytes(abi).unwrap(),
            pref: Align::from_bytes(pref).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// The endianness of the target architecture.
pub enum Endianess {
    /// Little-endian.
    Little,

    /// Big-endian.
    Big,
}

#[derive(Debug)]
/// Describes the target platform's data layout, including type alignments, pointer size,
/// and other ABI-related information used during code generation.
///
/// It includes alignment requirements for integer, float, and vector types, as well as
/// general properties such as pointer size and aggregate alignment.
pub struct TargetDataLayout {
    /// The endianness of the target architecture.
    pub endianess: Endianess,

    // Integer type alignments
    pub i1_align: AbiAndPrefAlign,
    pub i8_align: AbiAndPrefAlign,
    pub i16_align: AbiAndPrefAlign,
    pub i32_align: AbiAndPrefAlign,
    pub i64_align: AbiAndPrefAlign,
    pub i128_align: AbiAndPrefAlign,

    // Floating point type alignments
    pub f16_align: AbiAndPrefAlign,
    pub f32_align: AbiAndPrefAlign,
    pub f64_align: AbiAndPrefAlign,
    pub f128_align: AbiAndPrefAlign,

    /// The size of pointers in bytes.
    pub pointer_size: u64,

    /// The ABI and preferred alignment for pointers.
    pub pointer_align: AbiAndPrefAlign,

    /// The minimum and preferred alignment for aggregate types (e.g., structs, arrays).
    pub aggregate_align: AbiAndPrefAlign,

    /// Alignments for vector types.
    pub vector_align: Vec<(Size, AbiAndPrefAlign)>,

    /// An identifier that specifies the address space that some operation
    /// should operate on. Special address spaces have an effect on code generation,
    /// depending on the target and the address spaces it implements.
    pub instruction_address_space: AddressSpace,
}

impl Default for TargetDataLayout {
    fn default() -> Self {
        TargetDataLayout {
            endianess: Endianess::Big,
            i1_align: AbiAndPrefAlign::new(8, 8),
            i8_align: AbiAndPrefAlign::new(8, 8),
            i16_align: AbiAndPrefAlign::new(16, 16),
            i32_align: AbiAndPrefAlign::new(32, 32),
            i64_align: AbiAndPrefAlign::new(32, 64),
            i128_align: AbiAndPrefAlign::new(32, 64),
            f16_align: AbiAndPrefAlign::new(16, 16),
            f32_align: AbiAndPrefAlign::new(32, 32),
            f64_align: AbiAndPrefAlign::new(64, 64),
            f128_align: AbiAndPrefAlign::new(128, 128),
            pointer_size: 64,
            pointer_align: AbiAndPrefAlign::new(64, 64),
            aggregate_align: AbiAndPrefAlign::new(0, 64),
            vector_align: vec![
                (Size::from_bits(64), AbiAndPrefAlign::new(64, 64)),
                (Size::from_bits(128), AbiAndPrefAlign::new(128, 128)),
            ],
            instruction_address_space: AddressSpace::DATA,
        }
    }
}

impl TargetDataLayout {
    #[instrument]
    pub fn new() -> Self {
        let target_data_layout = TargetDataLayout::default();
        info!("TargetDataLayout created: {:?}", target_data_layout);
        target_data_layout
    }

    /// For example, for x86_64-unknown-linux-gnu, the data layout string could be:
    /// `e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128`
    pub fn into_llvm_datalayout_string(&self) -> String {
        let format_align = |name: &str, align: &AbiAndPrefAlign| {
            format!("-{}:{}:{}", name, align.abi.bytes(), align.pref.bytes())
        };

        let mut s = String::new();

        // Add endianess
        s.push(if self.endianess == Endianess::Little {
            'e'
        } else {
            'E'
        });

        // Add pointer and integer alignments
        s.push_str(&format!(
            "-p:{}:{}:{}",
            self.pointer_size,
            self.pointer_align.abi.bytes(),
            self.pointer_align.pref.bytes()
        ));

        // Format for integer types
        s.push_str(&format_align("i1", &self.i1_align));
        s.push_str(&format_align("i8", &self.i8_align));
        s.push_str(&format_align("i16", &self.i16_align));
        s.push_str(&format_align("i32", &self.i32_align));
        s.push_str(&format_align("i64", &self.i64_align));
        s.push_str(&format_align("i128", &self.i128_align));

        // Format for floating point types
        s.push_str(&format_align("f16", &self.f16_align));
        s.push_str(&format_align("f32", &self.f32_align));
        s.push_str(&format_align("f64", &self.f64_align));
        s.push_str(&format_align("f128", &self.f128_align));

        // Aggregate alignment
        s.push_str(&format_align("a", &self.aggregate_align));

        // Vector alignments
        for (size, align) in &self.vector_align {
            s.push_str(&format!(
                "-v{}:{}:{}",
                size.bytes(),
                align.abi.bytes(),
                align.pref.bytes()
            ));
        }

        // Instruction address space
        s.push_str(&format!("-P{}", u32::from(&self.instruction_address_space)));

        s
    }

    fn into_cranelift_datalayout_string(&self) -> String {
        unimplemented!()
    }

    fn into_gcc_datalayout_string(&self) -> String {
        unimplemented!()
    }

    // /// Parse data layout from an [llvm data layout string](https://llvm.org/docs/LangRef.html#data-layout)
    // /// For example, for x86_64-unknown-linux-gnu, the data layout string is:
    // /// `e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128`
    // pub fn parse_from_llvm_datalayout_string<'a>(
    //     input: &'a str,
    // ) -> Result<TargetDataLayout, TargetDataLayoutErrors<'a>> {
    //     // Parse an address space index from a string.
    //     let parse_address_space = |s: &'a str, cause: &'a str| {
    //         s.parse::<u32>().map(AddressSpace).map_err(|err| {
    //             TargetDataLayoutErrors::InvalidAddressSpace { addr_space: s, cause, err }
    //         })
    //     };
    //
    //     // Parse a bit count from a string.
    //     let parse_bits = |s: &'a str, kind: &'a str, cause: &'a str| {
    //         s.parse::<u64>().map_err(|err| TargetDataLayoutErrors::InvalidBits {
    //             kind,
    //             bit: s,
    //             cause,
    //             err,
    //         })
    //     };
    //
    //     // Parse a size string.
    //     let parse_size =
    //         |s: &'a str, cause: &'a str| parse_bits(s, "size", cause).map(Size::from_bits);
    //
    //     // Parse an alignment string.
    //     let parse_align = |s: &[&'a str], cause: &'a str| {
    //         if s.is_empty() {
    //             return Err(TargetDataLayoutErrors::MissingAlignment { cause });
    //         }
    //         let align_from_bits = |bits| {
    //             Align::from_bits(bits)
    //                 .map_err(|err| TargetDataLayoutErrors::InvalidAlignment { cause, err })
    //         };
    //         let abi = parse_bits(s[0], "alignment", cause)?;
    //         let pref = s.get(1).map_or(Ok(abi), |pref| parse_bits(pref, "alignment", cause))?;
    //         Ok(AbiAndPrefAlign { abi: align_from_bits(abi)?, pref: align_from_bits(pref)? })
    //     };
    //
    //     let mut dl = TargetDataLayout::default();
    //     let mut i128_align_src = 64;
    //     for spec in input.split('-') {
    //         let spec_parts = spec.split(':').collect::<Vec<_>>();
    //
    //         match &*spec_parts {
    //             ["e"] => dl.endian = Endian::Little,
    //             ["E"] => dl.endian = Endian::Big,
    //             [p] if p.starts_with('P') => {
    //                 dl.instruction_address_space = parse_address_space(&p[1..], "P")?
    //             }
    //             ["a", a @ ..] => dl.aggregate_align = parse_align(a, "a")?,
    //             ["f16", a @ ..] => dl.f16_align = parse_align(a, "f16")?,
    //             ["f32", a @ ..] => dl.f32_align = parse_align(a, "f32")?,
    //             ["f64", a @ ..] => dl.f64_align = parse_align(a, "f64")?,
    //             ["f128", a @ ..] => dl.f128_align = parse_align(a, "f128")?,
    //             // FIXME(erikdesjardins): we should be parsing nonzero address spaces
    //             // this will require replacing TargetDataLayout::{pointer_size,pointer_align}
    //             // with e.g. `fn pointer_size_in(AddressSpace)`
    //             [p @ "p", s, a @ ..] | [p @ "p0", s, a @ ..] => {
    //                 dl.pointer_size = parse_size(s, p)?;
    //                 dl.pointer_align = parse_align(a, p)?;
    //             }
    //             [s, a @ ..] if s.starts_with('i') => {
    //                 let Ok(bits) = s[1..].parse::<u64>() else {
    //                     parse_size(&s[1..], "i")?; // For the user error.
    //                     continue;
    //                 };
    //                 let a = parse_align(a, s)?;
    //                 match bits {
    //                     1 => dl.i1_align = a,
    //                     8 => dl.i8_align = a,
    //                     16 => dl.i16_align = a,
    //                     32 => dl.i32_align = a,
    //                     64 => dl.i64_align = a,
    //                     _ => {}
    //                 }
    //                 if bits >= i128_align_src && bits <= 128 {
    //                     // Default alignment for i128 is decided by taking the alignment of
    //                     // largest-sized i{64..=128}.
    //                     i128_align_src = bits;
    //                     dl.i128_align = a;
    //                 }
    //             }
    //             [s, a @ ..] if s.starts_with('v') => {
    //                 let v_size = parse_size(&s[1..], "v")?;
    //                 let a = parse_align(a, s)?;
    //                 if let Some(v) = dl.vector_align.iter_mut().find(|v| v.0 == v_size) {
    //                     v.1 = a;
    //                     continue;
    //                 }
    //                 // No existing entry, add a new one.
    //                 dl.vector_align.push((v_size, a));
    //             }
    //             _ => {} // Ignore everything else.
    //         }
    //     }
    //     Ok(dl)
    // }
}

#[derive(Debug)]
/// Represents a target triple, which uniquely identifies a compilation target.
///
/// A target triple is a string that encodes information about the target architecture,
/// vendor, operating system, environment, and ABI. This is commonly used to select
/// appropriate code generation strategies, linkers, standard libraries, and target-specific
/// configurations.
///
/// Example: `"x86_64-unknown-linux-gnu"`
///
/// Each component of the triple is stored separately for easier access and manipulation.
pub struct TargetTriple {
    /// The target architecture (e.g., "x86_64", "aarch64").
    pub arch: String,
    /// The target vendor (e.g., "unknown", "apple").
    pub vendor: String,
    /// The target operating system (e.g., "linux", "windows").
    pub os: String,
    /// The target environment or runtime (e.g., "gnu", "msvc", "musl").
    pub env: String,
    /// The ABI used on the target (e.g., "eabi", "gnu").
    pub abi: String,
}

impl TargetTriple {
    #[tracing::instrument]
    pub fn new(arch: &str, vendor: &str, os: &str, env: &str, abi: &str) -> Self {
        TargetTriple {
            arch: arch.to_string(),
            vendor: vendor.to_string(),
            os: os.to_string(),
            env: env.to_string(),
            abi: abi.to_string(),
        }
    }

    // ARCHITECTURE-VENDOR-OPERATING_SYSTEM-ENVIRONMENT
    pub fn into_llvm_triple_string(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.arch, self.vendor, self.os, self.env, self.abi
        )
    }

    pub fn into_cranelift_triple_string(&self) -> String {
        unimplemented!()
    }

    pub fn into_gcc_triple_string(&self) -> String {
        unimplemented!()
    }
}

/// Represents a type along with its size and alignment information.
///
/// This is commonly used during codegen and layout computation to reason about
/// how values should be represented in memory on the target platform.
pub struct TyAndLayout<T> {
    /// The type this layout refers to.
    ///
    /// This is usually a LIR type, but can be any type that has a size and alignment.
    pub ty: T,
    /// The size of the type in bytes.
    pub size: Size,
    /// The ABI and preferred alignment of the type.
    pub align: AbiAndPrefAlign,
}

#[derive(Debug)]
/// Describes the target configuration used during code generation.
///
/// This struct encapsulates information about the backend, data layout,
/// and optional target triple. It is used to drive architecture- and
/// platform-specific decisions throughout the compiler.
pub struct Target {
    /// The codegen backend to use.
    pub codegen_backend: BackendKind,
    /// The data layout configuration for the target, including type alignments,
    /// pointer size, and other ABI-relevant properties.
    pub data_layout: TargetDataLayout,
    /// The target triple string identifying the target architecture, vendor,
    /// operating system, and environment.
    ///
    /// If this is `None`, the target triple will not be set in the LLVM module,
    /// which may affect platform-specific codegen behavior or defaults.
    target_triple: Option<TargetTriple>,
}

impl Target {
    pub fn new(codegen_backend: BackendKind) -> Self {
        Target {
            data_layout: TargetDataLayout::new(),
            codegen_backend,
            target_triple: None,
        }
    }

    // TODO: make it better. Perhaps by using a specific TargetDataLayout for each
    // compiler backend.
    pub fn data_layout_string(&self) -> String {
        match self.codegen_backend {
            BackendKind::Llvm => self.data_layout.into_llvm_datalayout_string(),
            BackendKind::Cranelift => self.data_layout.into_cranelift_datalayout_string(),
            BackendKind::Gcc => self.data_layout.into_gcc_datalayout_string(),
        }
    }

    // TODO: make it better. Perhaps by using a specific TargetDataLayout for each
    // compiler backend.
    pub fn target_triple_string(&self) -> String {
        if self.target_triple.is_none() {
            return String::new();
        }

        match self.codegen_backend {
            BackendKind::Llvm => self
                .target_triple
                .as_ref()
                .unwrap()
                .into_llvm_triple_string(),
            BackendKind::Cranelift => self
                .target_triple
                .as_ref()
                .unwrap()
                .into_cranelift_triple_string(),
            BackendKind::Gcc => self
                .target_triple
                .as_ref()
                .unwrap()
                .into_gcc_triple_string(),
        }
    }
}
