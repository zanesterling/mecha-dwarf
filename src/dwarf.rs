use crate::macho;

#[derive(Debug)]
pub struct File {
    pub sections: Vec<Section>,
}

impl File {
    pub fn from(segment: macho::Segment64, bytes: &[u8]) -> Result<File, String> {
        Ok(File {
            sections: segment.sections.iter().map(|s| {
                let start = s.offset as usize;
                let end = start + s.size as usize;
                Section::from(s.sectname.as_str(), &bytes[start .. end])
            }).collect::<Result<Vec<Section>, String>>()?,
        })
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

        // TODO: Write a LEB128 parsing library.
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

    Unrecognized {
        name: String,
        contents: Vec<u8>,
    },
}

impl Section {
    pub fn from(name: &str, bytes: &[u8]) -> Result<Section, String> {
        match name {
            _ => Ok(Section::Unrecognized {
                name: name.to_string(),
                contents: bytes.to_vec(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct DebugLineFileEntry {
    // TODO: Fill out. Find docs in Section::DebugLine.
}

#[derive(Debug)]
pub struct CUHeader {
    pub unit_length: u32, // NOTE: In DWARF64, this would be 0xffffffff plus 8 bytes.
    pub version: u16,
    pub unit_type: u16,
    pub address_size: u8,
    pub debug_abbrev_offset: u32,
}

impl CUHeader {
    pub fn from(bytes: &[u8]) -> Result<CUHeader, String> {
        let unit_length         = u32::from_ne_bytes(bytes[ 0.. 4].try_into().unwrap());
        let version             = u16::from_ne_bytes(bytes[ 4.. 6].try_into().unwrap());
        let unit_type           = u16::from_ne_bytes(bytes[ 6.. 8].try_into().unwrap());
        let address_size        = bytes[8];
        let debug_abbrev_offset = u32::from_ne_bytes(bytes[ 9..13].try_into().unwrap());
        Ok(CUHeader {
            unit_length,
            version,
            unit_type,
            address_size,
            debug_abbrev_offset,
        })
    }
}
