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
