pub enum CodegenBackend {
    /// The LLVM backend.
    Llvm,

    /// The Cranelift backend.
    Cranelift,
}

pub struct AbiSize(pub u64);
pub struct AbiAlign(pub u64);

pub enum Endianess {
    /// Little-endian.
    Little,

    /// Big-endian.
    Big,
}

pub struct TargetDataLayout {
    pub endianess: Endianess,
    pub i1_align: AbiAlign,
    pub i8_align: AbiAlign,
    pub i16_align: AbiAlign,
    pub i32_align: AbiAlign,
    pub i64_align: AbiAlign,
    pub i128_align: AbiAlign,
    pub f16_align: AbiAlign,
    pub f32_align: AbiAlign,
    pub f64_align: AbiAlign,
    pub f128_align: AbiAlign,

    /// The size of pointers in bytes.
    pub pointer_size: u64,

    pub pointer_align: AbiAlign,
    pub aggregate_align: AbiAlign,

    /// Alignments for vector types.
    pub vector_align: Vec<(u64, AbiAlign)>,

    /// An identifier that specifies the address space that some operation
    /// should operate on. Special address spaces have an effect on code generation,
    /// depending on the target and the address spaces it implements.
    ///
    /// When `0`, which is the default address space, corresponds to the data space.
    pub instruction_address_space: u32,
}

impl Default for TargetDataLayout {
    fn default() -> Self {
        TargetDataLayout {
            endianess: Endianess::Big,
            i1_align: AbiAlign(8),
            i8_align: AbiAlign(8),
            i16_align: AbiAlign(16),
            i32_align: AbiAlign(32),
            i64_align: AbiAlign(32),
            i128_align: AbiAlign(32),
            f16_align: AbiAlign(16),
            f32_align: AbiAlign(32),
            f64_align: AbiAlign(64),
            f128_align: AbiAlign(128),
            pointer_size: 64,
            pointer_align: AbiAlign(64),
            aggregate_align: AbiAlign(0),
            vector_align: vec![(64, AbiAlign(64)), (128, AbiAlign(128))],
            instruction_address_space: 0,
        }
    }
}

pub struct TyAndLayout<T> {
    pub ty: T,
    pub size: AbiSize,
    pub align: AbiAlign,
}

impl TargetDataLayout {
    pub fn new(codegen_backend: CodegenBackend) -> Self {
        match codegen_backend {
            CodegenBackend::Llvm => todo!(),
            CodegenBackend::Cranelift => unimplemented!(),
        }
    }

    // /// Parse data layout from an [llvm data layout string](https://llvm.org/docs/LangRef.html#data-layout)
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
