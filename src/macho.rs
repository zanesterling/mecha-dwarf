use std::mem;

#[derive(Debug)]
pub struct Header {
    magic: u32,
    cpu_type: u32,
    cpu_subtype: u32,
    file_type: u32,
    loads_count: u32,
    loads_size: u32,
    flags: u32,
    reserved: u32,
}

impl Header {
    pub fn from(bytes: &[u8]) -> Header {
        unsafe {
            mem::transmute_copy::<[u8; 32], Header>(
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

#[derive(Debug)]
pub struct File {
    header: Header,
}
