use std::mem;

#[derive(Debug)]
pub struct File {
    pub header: Header,
    pub load_commands: Vec<LoadCommand>,
}

impl File {
    pub fn from(bytes: &[u8]) -> Result<File, String> {
        let header = Header::from_bytes(&bytes[0..32])?;
        let mut bytes_read = 32;
        let load_commands = {
            let start_of_loads = bytes_read;
            let mut vec: Vec<LoadCommand> = vec![];
            for _ in 0..header.loads_count {
                let (load, read) = LoadCommand::from(&bytes[bytes_read..])?;
                vec.push(load);
                bytes_read += read;
            }
            let loads_size = bytes_read - start_of_loads;
            if loads_size != header.loads_size.try_into().unwrap() {
                return Err(format!("expected loads to be {}B, but instead found {}B",
                        header.loads_size, loads_size));
            }
            vec
        };
        Ok(File {
            header,
            load_commands,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    pub cpu_type: CpuType,
    pub is_64_bit: bool,
    pub file_type: FileType,
    pub loads_count: u32,
    pub loads_size: u32,
    // TODO: Add flag: Flags, and deal with transmuting.
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Result<Header, String> {
        Self::from_header(RawHeader::from(bytes))
    }

    pub fn from_header(raw: RawHeader) -> Result<Header, String> {
        let is_64_bit = (0x01000000 & raw.cpu_type) != 0;
        match raw.magic {
            0xfeedface if !is_64_bit => {},
            0xfeedfacf if  is_64_bit => {},
            magic if is_64_bit => {
                return Err(format!("arch is 64-bit, but magic number is {:#010x}", magic));
            }
            magic => {
                return Err(format!("arch is 32-bit, but magic number is {:#010x}", magic));
            }
        }
        Ok(Header {
            cpu_type: CpuType::from(raw.cpu_type, raw.cpu_subtype)?,
            is_64_bit: is_64_bit,
            file_type: FileType::from(raw.file_type)
                .ok_or(format!("bad file type: {}", raw.file_type))?,
            loads_count: raw.loads_count,
            loads_size: raw.loads_size,
        })
    }
}

#[derive(Debug)]
pub enum CpuType {
    Vax,
    Romp,
    Ns32032,
    NS32332,
    MC680x0,
    X86(X86Subtype),
    Mips,
    Ns32352,
    Mc98000,
    Hppa,
    Arm(ArmSubtype),
    Mc88000,
    Sparc,
    I860BigEndian,
    I860LittleEndian,
    Rs6000,
    PowerPC,
}

impl CpuType {
    fn from(cpu_type: u32, cpu_subtype: u32)
        -> Result<CpuType, String> {
        match 0xFF & cpu_type {
            0x01 => Ok(CpuType::Vax),
            0x02 => Ok(CpuType::Romp),
            0x04 => Ok(CpuType::Ns32032),
            0x05 => Ok(CpuType::NS32332),
            0x06 => Ok(CpuType::MC680x0),
            0x07 => X86Subtype::from(cpu_subtype)
                .ok_or(format!("bad cpu subtype: {}", cpu_subtype))
                .map(CpuType::X86),
            0x08 => Ok(CpuType::Mips),
            0x09 => Ok(CpuType::Ns32352),
            0x0A => Ok(CpuType::Mc98000),
            0x0B => Ok(CpuType::Hppa),
            0x0C => ArmSubtype::from(cpu_subtype)
                .ok_or(format!("bad cpu subtype: {}", cpu_subtype))
                .map(CpuType::Arm),
            0x0D => Ok(CpuType::Mc88000),
            0x0E => Ok(CpuType::Sparc),
            0x0F => Ok(CpuType::I860BigEndian),
            0x10 => Ok(CpuType::I860LittleEndian),
            0x11 => Ok(CpuType::Rs6000),
            0x12 => Ok(CpuType::PowerPC),
            _ => Err(format!("bad cpu type: {}", cpu_type)),
        }
    }
}

#[derive(Debug)]
pub enum X86Subtype {
    AllX86,
    I486OrNewer,
    I486SXOrNewer,
    PentiumM5OrNewer,
    CeleronOrNewer,
    CeleronMobileOrNewer,
    Pentium3OrNewer,
    Pentium3MOrNewer,
    Pentium3XEONOrNewer,
    Pentium4OrNewer,
    ItaniumOrNewer,
    Itanium2OrNewer,
    XEONOrNewer,
    XEONMPOrNewer,
}

impl X86Subtype {
    fn from(word: u32) -> Option<X86Subtype> {
        match word {
            0x03 => Some(X86Subtype::AllX86),
            0x04 => Some(X86Subtype::I486OrNewer),
            0x84 => Some(X86Subtype::I486SXOrNewer),
            0x56 => Some(X86Subtype::PentiumM5OrNewer),
            0x67 => Some(X86Subtype::CeleronOrNewer),
            0x77 => Some(X86Subtype::CeleronMobileOrNewer),
            0x08 => Some(X86Subtype::Pentium3OrNewer),
            0x18 => Some(X86Subtype::Pentium3MOrNewer),
            0x28 => Some(X86Subtype::Pentium3XEONOrNewer),
            0x0A => Some(X86Subtype::Pentium4OrNewer),
            0x0B => Some(X86Subtype::ItaniumOrNewer),
            0x1B => Some(X86Subtype::Itanium2OrNewer),
            0x0C => Some(X86Subtype::XEONOrNewer),
            0x1C => Some(X86Subtype::XEONMPOrNewer),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ArmSubtype {
    AllArm,
    ArmA500ARCHOrNewer,
    ArmA500OrNewer,
    ArmA440OrNewer,
    ArmM4OrNewer,
    ArmV4TOrNewer,
    ArmV6OrNewer,
    ArmV5TEJOrNewer,
    ArmXSCALEOrNewer,
    ArmV7OrNewer,
    ArmV7FOrNewer,
    ArmV7SOrNewer,
    ArmV7KOrNewer,
    ArmV8OrNewer,
    ArmV6MOrNewer,
    ArmV7MOrNewer,
    ArmV7EMOrNewer,
}

impl ArmSubtype {
    fn from(word: u32) -> Option<ArmSubtype> {
        match word {
            0x00 => Some(ArmSubtype::AllArm),
            0x01 => Some(ArmSubtype::ArmA500ARCHOrNewer),
            0x02 => Some(ArmSubtype::ArmA500OrNewer),
            0x03 => Some(ArmSubtype::ArmA440OrNewer),
            0x04 => Some(ArmSubtype::ArmM4OrNewer),
            0x05 => Some(ArmSubtype::ArmV4TOrNewer),
            0x06 => Some(ArmSubtype::ArmV6OrNewer),
            0x07 => Some(ArmSubtype::ArmV5TEJOrNewer),
            0x08 => Some(ArmSubtype::ArmXSCALEOrNewer),
            0x09 => Some(ArmSubtype::ArmV7OrNewer),
            0x0A => Some(ArmSubtype::ArmV7FOrNewer),
            0x0B => Some(ArmSubtype::ArmV7SOrNewer),
            0x0C => Some(ArmSubtype::ArmV7KOrNewer),
            0x0D => Some(ArmSubtype::ArmV8OrNewer),
            0x0E => Some(ArmSubtype::ArmV6MOrNewer),
            0x0F => Some(ArmSubtype::ArmV7MOrNewer),
            0x10 => Some(ArmSubtype::ArmV7EMOrNewer),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum FileType {
    RelocatableObj,
    DemandPagedExe,
    FixedVmSharedLib,
    Core,
    PreloadedExe,
    DynamicallyBoundSharedLib,
    DynamicLinkEditor,
    DynamicallyBoundBundle,
    SharedLibraryStub, // Stub for static linking only, no section contents.
    CompanionDebugOnly,
    X8664Kexts,
    CompositeMacho,
}

impl FileType {
    pub fn from(word: u32) -> Option<FileType> {
        match word {
            0x01 => Some(FileType::RelocatableObj),
            0x02 => Some(FileType::DemandPagedExe),
            0x03 => Some(FileType::FixedVmSharedLib),
            0x04 => Some(FileType::Core),
            0x05 => Some(FileType::PreloadedExe),
            0x06 => Some(FileType::DynamicallyBoundSharedLib),
            0x07 => Some(FileType::DynamicLinkEditor),
            0x08 => Some(FileType::DynamicallyBoundBundle),
            0x09 => Some(FileType::SharedLibraryStub),
            0x0A => Some(FileType::CompanionDebugOnly),
            0x0B => Some(FileType::X8664Kexts),
            0x0C => Some(FileType::CompositeMacho),
            _ => None,
        }
    }
}

use bitflags::bitflags;
bitflags! {
    pub struct Flags: u32 {
        // The object file has no undefined references.
        const NO_UNDEFINED_REFERENCES = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        // The object file is the output of an incremental link against
        // a base file and can't be link edited again.
        const INCREMENTAL_LINK        = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        // The object file is input for the dynamic linker and can't be
        // statically link edited again.
        const DYNAMIC_LINKER_INPUT    = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        // The object file's undefined references are bound by the
        // dynamic linker when loaded.
        const DYNAMIC_LINKER_BINDREF  = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        // The file has its dynamic undefined references prebound.
        const DYN_UNDEF_REFS_PREBOUND = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        // The file has its read-only and write-only segments split.
        const SPLIT_READWRITE_ONLY    = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        // The shard library init routine is to be run lazily via
        // catching  memory faults to its writeable segments (obsolete).
        const LAZY_SHAREDLIB_INIT     = 0b0000_0000_0000_0000_0000_0000_0100_0000;
        // The image is using two-level namespace bindings.
        const TWO_LEVEL_NS_BINDINGS   = 0b0000_0000_0000_0000_0000_0000_1000_0000;
        // The executable is forcing all images to use flat name space bindings.
        const FLAT_NS_BINDINGS        = 0b0000_0000_0000_0000_0000_0001_0000_0000;
        // This umbrella guarantees no multiple definitions of symbols in its
        // sub-images so the two-level namespapce hints can always be used.
        const NO_MULTI_DEF_IN_SUBIMGS = 0b0000_0000_0000_0000_0000_0010_0000_0000;
        // Do not have dyld notify the prebinding agent about this executable.
        const DYLD_DONT_NOTIFY        = 0b0000_0000_0000_0000_0000_0100_0000_0000;
        // The binary is not prebound but can have its prebinding redone.
        // Only used when MH_PREBOUND is not set.
        const CAN_REDO_PREBINDING     = 0b0000_0000_0000_0000_0000_1000_0000_0000;
        // TODO: Add the rest of the flags.
    }
}

#[derive(Debug)]
pub enum LoadCommand {
    SymbolTable {
        symoff: u32,   /* symbol table offset */
        nsyms: u32,    /* number of symbol table entries */
        stroff: u32,   /* string table offset */
        strsize: u32,  /* string table size in bytes */
    },

    Segment64,

    Uuid([u8; 16]),

    BuildVersion {
        platform: BuildPlatform,
        minos: u32,
        sdk: u32,
        tools: Vec<BuildToolVersion>,
    },
}

#[derive(Debug)]
pub enum BuildPlatform {
    MacOS,
    IOS,
    TVOS,
    WatchOS,
    Other(u32),
}

impl BuildPlatform {
    pub fn from(word: u32) -> BuildPlatform {
        match word {
            1 => BuildPlatform::MacOS,
            2 => BuildPlatform::IOS,
            3 => BuildPlatform::TVOS,
            4 => BuildPlatform::WatchOS,
            _ => BuildPlatform::Other(word),
        }
    }
}

#[derive(Debug)]
pub struct BuildToolVersion {
    pub tool: u32,
    pub version: u32,
}

impl LoadCommand {
    pub fn from(bytes: &[u8]) -> Result<(LoadCommand, usize), String> {
        if bytes.len() < 8 { return Err("ran out of bytes reading load command".to_string()); }
        let (type_bytes, bytes) = bytes.split_at(mem::size_of::<u32>());
        let ttype = u32::from_ne_bytes(type_bytes.try_into().unwrap());
        let (size_bytes, bytes) = bytes.split_at(mem::size_of::<u32>());
        let size = u32::from_ne_bytes(size_bytes.try_into().unwrap());

        if bytes.len() < size as usize {
            return Err("ran out of bytes reading load command".to_string());
        }
        match ttype {
            0x02 => Ok(LoadCommand::SymbolTable {
                symoff:  u32::from_ne_bytes(bytes[ 0.. 4].try_into().unwrap()),
                nsyms:   u32::from_ne_bytes(bytes[ 4.. 8].try_into().unwrap()),
                stroff:  u32::from_ne_bytes(bytes[ 8..12].try_into().unwrap()),
                strsize: u32::from_ne_bytes(bytes[12..16].try_into().unwrap()),
            }),

            0x19 => Ok(LoadCommand::Segment64), // TODO

            0x1b => Ok(LoadCommand::Uuid(bytes[0..16].try_into().unwrap())),

            0x32 => {
                let platform = BuildPlatform::from(u32::from_ne_bytes(bytes[0..4].try_into().unwrap()));
                let minos  = u32::from_ne_bytes(bytes[ 4.. 8].try_into().unwrap());
                let sdk    = u32::from_ne_bytes(bytes[ 8..12].try_into().unwrap());
                let ntools = u32::from_ne_bytes(bytes[12..16].try_into().unwrap());
                let expected_size = 0x18 + ntools * 8;
                if size != expected_size {
                    return Err(format!("BuildCommand is {}B, but should be {}B. possible corruption", size, expected_size));
                }
                let mut tools: Vec<BuildToolVersion> = vec![];
                let tool_bytes = &bytes[16..];
                for i in 0..ntools {
                    let i = i as usize;
                    tools.push(BuildToolVersion {
                        tool:    u32::from_ne_bytes(tool_bytes[8*i   ..8*i +4].try_into().unwrap()),
                        version: u32::from_ne_bytes(tool_bytes[8*i +4..8*i +8].try_into().unwrap()),
                    });
                }
                Ok(LoadCommand::BuildVersion {
                    platform,
                    minos,
                    sdk,
                    tools,
                })
            },
            _ => Err(format!("unrecognized load cmd type: {:#04x}", ttype)),
        }.map(|cmd| (cmd, size as usize))
    }
}

#[derive(Debug)]
pub struct RawHeader {
    pub magic: u32,
    pub cpu_type: u32,
    pub cpu_subtype: u32,
    pub file_type: u32,
    pub loads_count: u32,
    pub loads_size: u32,
    pub flags: u32,
    pub reserved: u32,
}

impl RawHeader {
    pub fn from(bytes: &[u8]) -> RawHeader {
        unsafe {
            mem::transmute_copy::<[u8; 32], RawHeader>(
                bytes[0..32].try_into().unwrap())
        }
    }

    pub fn show(&self) {
        println!("Header {{");
        println!("\tmagic:       {:#010x}", self.magic);
        println!("\tcpu_type:    {:#010x}", self.cpu_type);
        println!("\tcpu_subtype: {:#010x}", self.cpu_subtype);
        println!("\tfile_type:   {:#010x}", self.file_type);
        println!("\tloads_count: {:#010x}", self.loads_count);
        println!("\tloads_size:  {:#010x}", self.loads_size);
        println!("\tflags:       {:#010x}", self.flags);
        println!("\treserved:    {:#010x}", self.reserved);
        println!("}}");
    }
}
