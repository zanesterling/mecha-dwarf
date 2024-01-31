use crate::leb::*;
use crate::macho;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct File {
    pub sections: Vec<Section>,
}

impl File {
    pub fn from(segment: macho::Segment64, bytes: &[u8]) -> Result<File, String> {
        let mut sections: Vec<Section> = segment.sections.iter()
            .map(|sec| Section::Unrecognized {
                name: sec.sectname.clone(),
                contents: vec![],
            })
            .collect();

        // Parse the __debug_abbrev section first,
        // so that it can be used by __debug_info.
        let (i, debug_abbrev) = segment.sections.iter()
            .enumerate()
            .find(|(_, sec)| sec.sectname.as_str() == "__debug_abbrev")
            .ok_or("missing __debug_abbrev section")?;
        sections[i] =
            Self::macho_section_to_dwarf(&debug_abbrev, &bytes, &sections)?;

        for (i, sec) in segment.sections.iter().enumerate() {
            let start = sec.offset as usize;
            let end = start + sec.size as usize;
            let sec = Section::from(
                sec.sectname.as_str(), &bytes[start .. end], &sections)?;
            sections[i] = sec;
        }
        Ok(File {
            sections,
        })
    }

    fn macho_section_to_dwarf(
        sec: &macho::Section64, bytes: &[u8], others: &Vec<Section>
    ) -> Result<Section, String> {
        let start = sec.offset as usize;
        let end = start + sec.size as usize;
        Section::from(sec.sectname.as_str(), &bytes[start .. end], others)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for sec in self.sections.iter() {
            write!(f, "{}", sec)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Section {
    DebugLine {
        // The size in bytes of the line number information for this compilation
        // unit, not including the unit_length field itself.
        unit_length: u64,

        // A version number. This number is specific to the line number
        // information and is independent of the DWARF version number.
        version: u16,

        // The number of bytes following the header_length field to the
        // beginning of the first byte of the line number program itself.
        // In the 32-bit DWARF format, this is a 4-byte unsigned length;
        // in the 64-bit DWARF format, this field is an 8-byte unsigned length.
        header_length: u64,

        // The size in bytes of the smallest target machine instruction.
        // Line number program opcodes that alter the address and op_index
        // registers use this and maximum_operations_per_instruction in their
        // calculations.
        minimum_instruction_length: u8,

        // The maximum number of individual operations that may be encoded
        // in an instruction. Line number program opcodes that alter the address
        // and op_index registers use this and minimum_instruction_length in
        // their calculations.
        //
        // For non-VLIW architectures, this field is 1,
        // the op_index register is always 0,
        // and the operation pointer is simply the address register.
        maximum_operations_per_instruction: u8,

        // The initial value of the is_stmt register.
        //
        // A simple approach to building line number information when machine
        // instructions are emitted in an order corresponding to the source
        // program is to set default_is_stmt to “true” and to not change the
        // value of the is_stmt register within the line number program. One
        // matrix entry is produced for each line that has code generated for
        // it. The effect is that every entry in the matrix recommends the
        // beginning of each represented line as a breakpoint location. This is
        // the traditional practice for unoptimized code.
        //
        // A more sophisticated approach might involve multiple entries in the
        // matrix for a line number; in this case, at least one entry (often but
        // not necessarily only one) specifies a recommended breakpoint location
        // for the line number. DW_LNS_negate_stmt opcodes in the line number
        // program control which matrix entries constitute such a recommendation
        // and default_is_stmt might be either “true” or “false”. This approach
        // might be used as part of support for debugging optimized code.
        default_is_stmt: u8,

        // This parameter affects the meaning of the special opcodes.
        line_base: i8,

        // This parameter affects the meaning of the special opcodes.
        line_range: u8,

        // The number assigned to the first special opcode.
        //
        // Opcode base is typically one greater than the highest-numbered
        // standard opcode defined for the specified version of the line number
        // information (12 in DWARF Version 3 and Version 4, 9 in Version 2). If
        // opcode_base is less than the typical value, then standard opcode
        // numbers greater than or equal to the opcode base are not used in the
        // line number table of this unit (and the codes are treated as special
        // opcodes). If opcode_base is greater than the typical value, then the
        // numbers between that of the highest standard opcode and the first
        // special opcode (not inclusive) are used for vendor specific
        // extensions.
        opcode_base: u8,

        // This array specifies the number of LEB128 operands for each of the
        // standard opcodes. The first element of the array corresponds to the
        // opcode whose value is 1, and the last element corresponds to the
        // opcode whose value is opcode_base - 1.
        //
        // By increasing opcode_base, and adding elements to this array, new
        // standard opcodes can be added, while allowing consumers who do not
        // know about these new opcodes to be able to skip them.
        //
        // Codes for vendor specific extensions, if any, are described just like
        // standard opcodes.
        standard_opcode_lengths: Vec<u8>,

        // Entries in this sequence describe each path that was searched for
        // included source files in this compilation. (The paths include those
        // directories specified explicitly by the user for the compiler to
        // search and those the compiler searches without explicit direction.)
        // Each path entry is either a full path name or is relative to the
        // current directory of the compilation.
        //
        // The last entry is followed by a single null byte.
        //
        // The line number program assigns numbers to each of the file entries
        // in order, beginning with 1. The current directory of the compilation
        // is understood to be the zeroth entry and is not explicitly
        // represented.
        include_directories: Vec<String>,

        // Entries in this sequence describe source files that contribute to the
        // line number information for this compilation unit or is used in other
        // contexts, such as in a declaration coordinate or a macro file
        // inclusion. Each entry consists of the following values:
        //
        // - A null-terminated string containing the full or relative path name
        //   of a source file. If the entry contains a file name or relative
        //   path name, the file is located relative to either the compilation
        //   directory (as specified by the DW_AT_comp_dir attribute given in
        //   the compilation unit) or one of the directories listed in the
        //   include_directories section.
        // - An unsigned LEB128 number representing the directory index of a
        //   directory in the include_directories section.
        // - An unsigned LEB128 number representing the (implementation-defined)
        //   time of last modification for the file, or 0 if not available.
        // - An unsigned LEB128 number representing the length in bytes of the
        //   file, or 0 if not available.
        //
        // The last entry is followed by a single null byte.
        //
        // The directory index represents an entry in the include_directories
        // section. The index is 0 if the file was found in the current
        // directory of the compilation, 1 if it was found in the first
        // directory in the include_directories section, and so on. The
        // directory index is ignored for file names that represent full path
        // names.
        //
        // The primary source file is described by an entry whose path name
        // exactly matches that given in the DW_AT_name attribute in the
        // compilation unit, and whose directory is understood to be given by
        // the implicit entry with index 0.
        //
        // The line number program assigns numbers to each of the file entries
        // in order, beginning with 1, and uses those numbers instead of file
        // names in the file register.
        //
        // A compiler may generate a single null byte for the file names field
        // and define file names using the extended opcode DW_LNE_define_file.
        file_names: Vec<DebugLineFileEntry>,
    },

    DebugInfo {
        header: CUHeader,
        dies: Vec<DIE>,
    },

    DebugAbbrev {
        abbrevs: Vec<AbbrevDecl>,
    },

    Unrecognized {
        name: String,
        contents: Vec<u8>,
    },
}

impl Section {
    pub fn from(
        name: &str, bytes: &[u8], others: &Vec<Section>
    ) -> Result<Section, String> {
        match name {
            "__debug_info" => {
                let header = CUHeader::from(&bytes[0..11]);
                let offset = 11;
                let debug_abbrev = others.iter().filter_map(|sect|
                    match &sect {
                        Section::DebugAbbrev { abbrevs } => Some(abbrevs),
                        _ => None,
                    }
                ).next().ok_or("haven't parsed __debug_abbrev yet")?;
                // TODO: How do we know if there are multiple compilation units?
                let (die, _) = DIE::from(&bytes[offset..], debug_abbrev)?;
                Ok(Section::DebugInfo {
                    header,
                    dies: vec![die],
                })
            },

            "__debug_abbrev" => {
                let mut abbrevs = vec![];
                let mut offset = 0;
                loop {
                    let (code, _) = uleb128_decode(&bytes[offset..])?;
                    if code == 0 { break; }
                    let (abbr, size) = AbbrevDecl::from(&bytes[offset..])?;
                    offset += size;
                    abbrevs.push(abbr);
                }
                Ok(Section::DebugAbbrev {
                    abbrevs,
                })
            },

            _ => Ok(Section::Unrecognized {
                name: name.to_string(),
                contents: bytes.to_vec(),
            }),
        }
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Section::DebugAbbrev { abbrevs } => {
                write!(f, ".debug_abbrev contents:\n")?;
                for abbr in abbrevs {
                    write!(f, "[{}] {:?} DW_CHILDREN={}\n",
                        abbr.abbrev_code, abbr.tag, abbr.has_children)?;
                    for spec in abbr.attr_specs.iter() {
                        let name = format!("{:x?}", spec.name);
                        let form = format!("{:x?}", spec.form);
                        write!(f, "\t{:<20} {:<20}\n", name, form)?;
                    }
                    write!(f, "\n")?;
                }
            },

            Section::Unrecognized { name, contents } =>
                println!("Unrecognized {:16} {:#x} bytes", name, contents.len()),

            Section::DebugInfo { header, dies } => {
                write!(f, "{}\n", header)?;
                for die in dies.iter() {
                    write!(f, "{}\n", die)?;
                }
            },

            _ => write!(f, "{:#x?}", self)?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DebugLineFileEntry {
    // TODO: Fill out. Find docs in Section::DebugLine.
}

// Compile Unit Header
#[derive(Debug)]
pub struct CUHeader {
    // A 4-byte or 12-byte unsigned integer representing the length of the
    // .debug_info contribution for that compilation unit, not including the
    // length field itself. In the 32-bit DWARF format, this is a 4-byte
    // unsigned integer (which must be less than 0xfffffff0); in the 64-bit
    // DWARF format, this consists of the 4-byte value 0xffffffff followed by an
    // 8- byte unsigned integer that gives the actual length (see Section 7.4).
    pub unit_length: u32, // NOTE: In DWARF64, this would be 0xffffffff plus 8 bytes.

    // A 2-byte unsigned integer representing the version of the DWARF
    // information for the compilation unit (see Appendix F). The value in this
    // field is 4.
    pub version: u16,

    // A 4-byte or 8-byte unsigned offset into the .debug_abbrev section. This
    // offset associates the compilation unit with a particular set of debugging
    // information entry abbreviations. In the 32-bit DWARF format, this is a
    // 4-byte unsigned length; in the 64-bit DWARF format, this is an 8-byte
    // unsigned length (see Section 7.4).
    pub debug_abbrev_offset: u32, // NOTE: In DWARF64, this would be 0xffffffff plus 8 bytes.

    // A 1-byte unsigned integer representing the size in bytes of an address on
    // the target architecture. If the system uses segmented addressing, this
    // value represents the size of the offset portion of an address.
    pub address_size: u8,
}

impl CUHeader {
    // Consumes 11 bytes.
    pub fn from(bytes: &[u8]) -> CUHeader {
        let unit_length         = u32::from_ne_bytes(bytes[ 0.. 4].try_into().unwrap());
        let version             = u16::from_ne_bytes(bytes[ 4.. 6].try_into().unwrap());
        let debug_abbrev_offset = u32::from_ne_bytes(bytes[ 6.. 10].try_into().unwrap());
        let address_size        = bytes[10];
        CUHeader {
            unit_length,
            version,
            debug_abbrev_offset,
            address_size,
        }
    }
}

impl Display for CUHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "length = {:#010x?}, version = {:#06x?}, abbr_offset = {:#010x?}, address_size = {:#04x?}\n",
            self.unit_length, self.version, self.debug_abbrev_offset, self.address_size)
    }
}

// Debugging Information Entry
#[derive(Debug)]
pub struct DIE {
    pub tag: DIETag,
    pub attrs: Vec<DIEAttribute>,
    pub children: Vec<DIE>,
}

impl DIE {
    pub fn from(
        bytes: &[u8], abbrev_decls: &Vec<AbbrevDecl>
    ) -> Result<(DIE, usize), String> {
        let (abbr_code, size) = uleb128_decode(bytes)?;
        let decl = abbrev_decls.iter().find(|decl| decl.abbrev_code == abbr_code)
            .ok_or_else(|| format!("found no abbrev matching code: {:#x?}", abbr_code))?;
        let mut offset = size;

        // TODO: Parse the attributes of this DIE.
        let mut attrs: Vec<DIEAttribute> = vec![];
        for spec in decl.attr_specs.iter() {
            let (value, size) = AttrValue::from(&bytes[offset..], spec.form.clone())?;
            offset += size;
            attrs.push(DIEAttribute {
                name: spec.name.clone(),
                value,
            });
        }

        let children = if decl.has_children {
            let (children, size) = Self::nfrom(&bytes[offset..], abbrev_decls)?;
            offset += size;
            children
        } else { vec![] };
        Ok((
            DIE {
                tag: decl.tag,
                attrs,
                children,
            },
            offset,
        ))
    }

    pub fn nfrom(
        bytes: &[u8], abbrev_decls: &Vec<AbbrevDecl>
    ) -> Result<(Vec<DIE>, usize), String> {
        let mut dies = vec![];
        let mut offset = 0;
        loop {
            let (code, size) = uleb128_decode(&bytes[offset..])?;
            if code == 0 {
                offset += size;
                break;
            }
            let (die, size) = Self::from(&bytes[offset..], abbrev_decls)?;
            dies.push(die);
            offset += size;
        }
        Ok((dies, offset))
    }
}

impl Display for DIE {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "DW_TAG_{:?}\n", self.tag)?;
        for attr in self.attrs.iter() {
            let name = format!("{:x?}", attr.name);
            write!(f, "\tDW_AT_{:<20} {:x?}\n", name, attr.value)?;
        }
        for child in self.children.iter() {
            write!(f, "\n{}", child)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DIETag {
    ArrayType,
    ClassType,
    EntryPoint,
    EnumerationType,
    FormalParameter,
    ImportedDeclaration,
    Label,
    LexicalBlock,
    Member,
    PointerType,
    ReferenceType,
    CompileUnit,
    StringType,
    StructureType,
    SubroutineType,
    Typedef,
    UnionType,
    UnspecifiedParameters,
    Variant,
    CommonBlock,
    CommonInclusion,
    Inheritance,
    InlinedSubroutine,
    Module,
    PtrToMemberType,
    SetType,
    SubrangeType,
    WithStmt,
    AccessDeclaration,
    BaseType,
    CatchBlock,
    ConstType,
    Constant,
    Enumerator,
    FileType,
    Friend,
    Namelist,
    NamelistItem,
    PackedType,
    Subprogram,
    TemplateTypeParameter,
    TemplateValueParameter,
    ThrownType,
    TryBlock,
    VariantPart,
    Variable,
    VolatileType,
    DwarfProcedure,
    RestrictType,
    InterfaceType,
    Namespace,
    ImportedModule,
    UnspecifiedType,
    PartialUnit,
    ImportedUnit,
    Condition,
    SharedType,
    TypeUnit,
    RvalueReferenceType,
    TemplateAlias,
    LoUser,
    HiUser,
}

impl DIETag {
    // TODO: this u16 is the output of LEB128 decoding. Arguably should be
    // size-invariant.
    pub fn from(value: u64) -> Result<DIETag, String> {
        match value {
           0x01   => Ok(DIETag::ArrayType),
           0x02   => Ok(DIETag::ClassType),
           0x03   => Ok(DIETag::EntryPoint),
           0x04   => Ok(DIETag::EnumerationType),
           0x05   => Ok(DIETag::FormalParameter),
           0x08   => Ok(DIETag::ImportedDeclaration),
           0x0a   => Ok(DIETag::Label),
           0x0b   => Ok(DIETag::LexicalBlock),
           0x0d   => Ok(DIETag::Member),
           0x0f   => Ok(DIETag::PointerType),
           0x10   => Ok(DIETag::ReferenceType),
           0x11   => Ok(DIETag::CompileUnit),
           0x12   => Ok(DIETag::StringType),
           0x13   => Ok(DIETag::StructureType),
           0x15   => Ok(DIETag::SubroutineType),
           0x16   => Ok(DIETag::Typedef),
           0x17   => Ok(DIETag::UnionType),
           0x18   => Ok(DIETag::UnspecifiedParameters),
           0x19   => Ok(DIETag::Variant),
           0x1a   => Ok(DIETag::CommonBlock),
           0x1b   => Ok(DIETag::CommonInclusion ),
           0x1c   => Ok(DIETag::Inheritance),
           0x1d   => Ok(DIETag::InlinedSubroutine),
           0x1e   => Ok(DIETag::Module),
           0x1f   => Ok(DIETag::PtrToMemberType),
           0x20   => Ok(DIETag::SetType),
           0x21   => Ok(DIETag::SubrangeType),
           0x22   => Ok(DIETag::WithStmt),
           0x23   => Ok(DIETag::AccessDeclaration),
           0x24   => Ok(DIETag::BaseType),
           0x25   => Ok(DIETag::CatchBlock),
           0x26   => Ok(DIETag::ConstType),
           0x27   => Ok(DIETag::Constant),
           0x28   => Ok(DIETag::Enumerator),
           0x29   => Ok(DIETag::FileType),
           0x2a   => Ok(DIETag::Friend),
           0x2b   => Ok(DIETag::Namelist),
           0x2c   => Ok(DIETag::NamelistItem),
           0x2d   => Ok(DIETag::PackedType),
           0x2e   => Ok(DIETag::Subprogram),
           0x2f   => Ok(DIETag::TemplateTypeParameter),
           0x30   => Ok(DIETag::TemplateValueParameter),
           0x31   => Ok(DIETag::ThrownType),
           0x32   => Ok(DIETag::TryBlock),
           0x33   => Ok(DIETag::VariantPart),
           0x34   => Ok(DIETag::Variable),
           0x35   => Ok(DIETag::VolatileType),
           0x36   => Ok(DIETag::DwarfProcedure),
           0x37   => Ok(DIETag::RestrictType),
           0x38   => Ok(DIETag::InterfaceType),
           0x39   => Ok(DIETag::Namespace),
           0x3a   => Ok(DIETag::ImportedModule),
           0x3b   => Ok(DIETag::UnspecifiedType),
           0x3c   => Ok(DIETag::PartialUnit),
           0x3d   => Ok(DIETag::ImportedUnit),
           0x3f   => Ok(DIETag::Condition),
           0x40   => Ok(DIETag::SharedType),
           0x41   => Ok(DIETag::TypeUnit),
           0x42   => Ok(DIETag::RvalueReferenceType),
           0x43   => Ok(DIETag::TemplateAlias),
           0x4080 => Ok(DIETag::LoUser),
           0xffff => Ok(DIETag::HiUser),
           _ => Err(format!("bad DIE tag {:#x}", value)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DIEAttribute {
    pub name: AttrName,
    pub value: AttrValue,
}

impl DIEAttribute {
}

#[derive(Clone, Debug)]
pub enum AttrValue {
    Address(u64),
    Constant(u64),
    ExprLoc(Vec<u8>), // Holds an expression or location description.
    Flag(bool),
    MacPtr(u64),
    OffsetReference(u64),
    StrP(u64),
    Unimplemented(AttrForm),
}

impl AttrValue {
    pub fn from(
        bytes: &[u8], form: AttrForm
    ) -> Result<(AttrValue, usize), String> {
        match form {
            AttrForm::Addr => {
                // FIXME: Address size is set in the unit header.
                let x = u64::from_ne_bytes(bytes[0..8].try_into().unwrap());
                Ok((AttrValue::Address(x), 8))
            },
            AttrForm::Data1 => Ok((AttrValue::Constant(bytes[0] as u64), 1)),
            AttrForm::Data2 => {
                let x = u16::from_ne_bytes(bytes[0..2].try_into().unwrap());
                Ok((AttrValue::Constant(x as u64), 2))
            },
            AttrForm::Data4 => {
                let x = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
                Ok((AttrValue::Constant(x as u64), 4))
            },
            AttrForm::ExprLoc => {
                let (len, size) = uleb128_decode(bytes)?;
                let (len, size) = (len as usize, size as usize);
                Ok((AttrValue::ExprLoc(bytes[size..size+len].to_vec()), len + size))
            },
            AttrForm::Flag => Ok((AttrValue::Flag(bytes[0] != 0), 1)),
            AttrForm::FlagPresent => Ok((AttrValue::Flag(true), 0)),
            AttrForm::Ref1 => Ok((AttrValue::OffsetReference(bytes[0] as u64), 1)),
            AttrForm::Ref2 => {
                let x = u16::from_ne_bytes(bytes[0..2].try_into().unwrap());
                Ok((AttrValue::OffsetReference(x as u64), 2))
            },
            AttrForm::Ref4 => {
                let x = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
                Ok((AttrValue::OffsetReference(x as u64), 4))
            },
            AttrForm::Ref8 => {
                let x = u32::from_ne_bytes(bytes[0..8].try_into().unwrap());
                Ok((AttrValue::OffsetReference(x as u64), 8))
            },
            AttrForm::SecOffset => {
                let x = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
                Ok((AttrValue::MacPtr(x as u64), 4))
            },
            AttrForm::StrP => {
                let x = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
                Ok((AttrValue::StrP(x as u64), 4))
            },
            _ => Ok((AttrValue::Unimplemented(form), 0)),
        }
    }
}

#[derive(Debug)]
pub struct AbbrevDecl {
    pub abbrev_code: u64,
    pub tag: DIETag,
    pub has_children: bool,
    pub attr_specs: Vec<AttrSpec>,
}

impl AbbrevDecl {
    pub fn from(bytes: &[u8]) -> Result<(AbbrevDecl, usize), String> {
        let mut offset = 0;
        let (abbrev_code, code_size) = uleb128_decode(bytes)?;
        offset += code_size;
        let (tag, code_size) = uleb128_decode(&bytes[offset..])?;
        offset += code_size;
        let has_children = match bytes[offset] {
            0 => Ok(false),
            1 => Ok(true),
            x => Err(format!("bad DW_CHILDREN value, {}", x)),
        }?;
        offset += 1;
        let mut attr_specs = vec![];
        loop {
            let (name, leb_size) = uleb128_decode(&bytes[offset..])?;
            offset += leb_size;
            let (form, leb_size) = uleb128_decode(&bytes[offset..])?;
            offset += leb_size;
            if name == 0 && form == 0 { break; }
            attr_specs.push(AttrSpec {
                name: AttrName::from(name),
                form: AttrForm::from(form),
            });
        }
        Ok((
            AbbrevDecl {
                abbrev_code,
                tag: DIETag::from(tag)?,
                has_children,
                attr_specs,
            },
            offset,
        ))
    }
}

#[derive(Debug)]
pub struct AttrSpec {
    pub name: AttrName,
    pub form: AttrForm,
}

#[derive(Clone, Debug)]
pub enum AttrName {
    Sibling,
    Location,
    Name,
    Ordering,
    ByteSize,
    BitOffset,
    BitSize,
    StmtList,
    LowPc,
    HighPc,
    Language,
    Discr,
    DiscrValue,
    Visibility,
    Import,
    StringLength,
    CommonReference,
    CompDir,
    ConstValue,
    ContainingType,
    DefaultValue,
    Inline,
    IsOptional,
    LowerBound,
    Producer,
    Prototyped,
    ReturnAddr,
    StartScope,
    BitStride,
    UpperBound,
    AbstractOrigin,
    Accessibility,
    AddressClass,
    Artificial,
    BaseTypes,
    CallingConvention,
    Count,
    DataMemberLocation,
    DeclColumn,
    DeclFile,
    DeclLine,
    Declaration,
    DiscrList,
    Encoding,
    External,
    FrameBase,
    Friend,
    IdentifierCase,
    MacroInfo,
    NamelistItem,
    Priority,
    Segment,
    Specification,
    StaticLink,
    Type,
    UseLocation,
    VariableParameter,
    Virtuality,
    VtableElemLocation,
    Allocated,
    Associated,
    DataLocation,
    ByteStride,
    EntryPc,
    UseUTF8,
    Extension,
    Ranges,
    Trampoline,
    CallColumn,
    CallFile,
    CallLine,
    Description,
    BinaryScale,
    DecimalScale,
    Small,
    DecimalSign,
    DigitCount,
    PictureString,
    Mutable,
    ThreadsScaled,
    Explicit,
    ObjectPointer,
    Endianity,
    Elemental,
    Pure,
    Recursive,
    Signature,
    MainSubprogram,
    DataBitOffset,
    ConstExpr,
    EnumClass,
    LinkageName,
    LoUser,
    HiUser,
    Unrecognized(u64),
}

impl AttrName {
    pub fn from(n: u64) -> AttrName {
        match n {
            0x01   => AttrName::Sibling,
            0x02   => AttrName::Location,
            0x03   => AttrName::Name,
            0x09   => AttrName::Ordering,
            0x0b   => AttrName::ByteSize,
            0x0c   => AttrName::BitOffset,
            0x0d   => AttrName::BitSize,
            0x10   => AttrName::StmtList,
            0x11   => AttrName::LowPc,
            0x12   => AttrName::HighPc,
            0x13   => AttrName::Language,
            0x15   => AttrName::Discr,
            0x16   => AttrName::DiscrValue,
            0x17   => AttrName::Visibility,
            0x18   => AttrName::Import,
            0x19   => AttrName::StringLength,
            0x1a   => AttrName::CommonReference,
            0x1b   => AttrName::CompDir,
            0x1c   => AttrName::ConstValue,
            0x1d   => AttrName::ContainingType,
            0x1e   => AttrName::DefaultValue,
            0x20   => AttrName::Inline,
            0x21   => AttrName::IsOptional,
            0x22   => AttrName::LowerBound,
            0x25   => AttrName::Producer,
            0x27   => AttrName::Prototyped,
            0x2a   => AttrName::ReturnAddr,
            0x2c   => AttrName::StartScope,
            0x2e   => AttrName::BitStride,
            0x2f   => AttrName::UpperBound,
            0x31   => AttrName::AbstractOrigin,
            0x32   => AttrName::Accessibility,
            0x33   => AttrName::AddressClass,
            0x34   => AttrName::Artificial,
            0x35   => AttrName::BaseTypes,
            0x36   => AttrName::CallingConvention,
            0x37   => AttrName::Count,
            0x38   => AttrName::DataMemberLocation,
            0x39   => AttrName::DeclColumn,
            0x3a   => AttrName::DeclFile,
            0x3b   => AttrName::DeclLine,
            0x3c   => AttrName::Declaration,
            0x3d   => AttrName::DiscrList,
            0x3e   => AttrName::Encoding,
            0x3f   => AttrName::External,
            0x40   => AttrName::FrameBase,
            0x41   => AttrName::Friend,
            0x42   => AttrName::IdentifierCase,
            0x43   => AttrName::MacroInfo,
            0x44   => AttrName::NamelistItem,
            0x45   => AttrName::Priority,
            0x46   => AttrName::Segment,
            0x47   => AttrName::Specification,
            0x48   => AttrName::StaticLink,
            0x49   => AttrName::Type,
            0x4a   => AttrName::UseLocation,
            0x4b   => AttrName::VariableParameter,
            0x4c   => AttrName::Virtuality,
            0x4d   => AttrName::VtableElemLocation,
            0x4e   => AttrName::Allocated,
            0x4f   => AttrName::Associated,
            0x50   => AttrName::DataLocation,
            0x51   => AttrName::ByteStride,
            0x52   => AttrName::EntryPc,
            0x53   => AttrName::UseUTF8,
            0x54   => AttrName::Extension,
            0x55   => AttrName::Ranges,
            0x56   => AttrName::Trampoline,
            0x57   => AttrName::CallColumn,
            0x58   => AttrName::CallFile,
            0x59   => AttrName::CallLine,
            0x5a   => AttrName::Description,
            0x5b   => AttrName::BinaryScale,
            0x5c   => AttrName::DecimalScale,
            0x5d   => AttrName::Small,
            0x5e   => AttrName::DecimalSign,
            0x5f   => AttrName::DigitCount,
            0x60   => AttrName::PictureString,
            0x61   => AttrName::Mutable,
            0x62   => AttrName::ThreadsScaled,
            0x63   => AttrName::Explicit,
            0x64   => AttrName::ObjectPointer,
            0x65   => AttrName::Endianity,
            0x66   => AttrName::Elemental,
            0x67   => AttrName::Pure,
            0x68   => AttrName::Recursive,
            0x69   => AttrName::Signature,
            0x6a   => AttrName::MainSubprogram,
            0x6b   => AttrName::DataBitOffset,
            0x6c   => AttrName::ConstExpr,
            0x6d   => AttrName::EnumClass,
            0x6e   => AttrName::LinkageName,
            0x2000 => AttrName::LoUser,
            0x3fff => AttrName::HiUser,
            n => AttrName::Unrecognized(n),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AttrForm {
    Addr,
    Block2,
    Block4,
    Data2,
    Data4,
    Data8,
    Stringg,
    Block,
    Block1,
    Data1,
    Flag,
    SData,
    StrP,
    Udata,
    RefAddr,
    Ref1,
    Ref2,
    Ref4,
    Ref8,
    RefUdata,
    Indirect,
    SecOffset,
    ExprLoc,
    FlagPresent,
    RefSig8,
    Unrecognized(u64),
}

impl AttrForm {
    pub fn from(n: u64) -> AttrForm {
        match n {
            0x01 => AttrForm::Addr,
            0x03 => AttrForm::Block2,
            0x04 => AttrForm::Block4,
            0x05 => AttrForm::Data2,
            0x06 => AttrForm::Data4,
            0x07 => AttrForm::Data8,
            0x08 => AttrForm::Stringg,
            0x09 => AttrForm::Block,
            0x0a => AttrForm::Block1,
            0x0b => AttrForm::Data1,
            0x0c => AttrForm::Flag,
            0x0d => AttrForm::SData,
            0x0e => AttrForm::StrP,
            0x0f => AttrForm::Udata,
            0x10 => AttrForm::RefAddr,
            0x11 => AttrForm::Ref1,
            0x12 => AttrForm::Ref2,
            0x13 => AttrForm::Ref4,
            0x14 => AttrForm::Ref8,
            0x15 => AttrForm::RefUdata,
            0x16 => AttrForm::Indirect,
            0x17 => AttrForm::SecOffset,
            0x18 => AttrForm::ExprLoc,
            0x19 => AttrForm::FlagPresent,
            0x20 => AttrForm::RefSig8,
            n => AttrForm::Unrecognized(n),
        }
    }
}
