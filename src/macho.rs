use std::mem;

#[derive(Debug)]
pub struct File {
    header: Header,
}

#[derive(Debug)]
pub struct Header {
    cpu_type: CpuType,
    is_64_bit: bool,
    file_type: FileType,
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Result<Header, String> {
        Self::from_header(RawHeader::from(bytes))
    }

    pub fn from_header(raw: RawHeader) -> Result<Header, String> {
        Ok(Header {
            cpu_type: CpuType::from(raw.cpu_type, raw.cpu_subtype)?,
            is_64_bit: (0x01000000 & raw.cpu_type) != 0,
            file_type: FileType::from(raw.file_type)
                .ok_or(format!("bad file type: {}", raw.file_type))?,
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
                .map(|t| CpuType::X86(t)),
            0x08 => Ok(CpuType::Mips),
            0x09 => Ok(CpuType::Ns32352),
            0x0A => Ok(CpuType::Mc98000),
            0x0B => Ok(CpuType::Hppa),
            0x0C => ArmSubtype::from(cpu_subtype)
                .ok_or(format!("bad cpu subtype: {}", cpu_subtype))
                .map(|t| CpuType::Arm(t)),
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

#[derive(Debug)]
pub struct RawHeader {
    magic: u32,
    cpu_type: u32,
    cpu_subtype: u32,
    file_type: u32,
    loads_count: u32,
    loads_size: u32,
    flags: u32,
    reserved: u32,
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
