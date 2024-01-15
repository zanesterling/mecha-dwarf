use std::fs::File;

use memmap::{ Mmap, MmapOptions };

mod macho;

fn main() {
    let path = {
        let mut args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            usage(args);
            std::process::exit(1);
        }
        args.swap_remove(1)
    };

    let mmap: Mmap = mmap_file(path)
        .unwrap_or_else(|e| {
            println!("{}", e);
            std::process::exit(1);
        });
    let macho = macho::File::from(&mmap[..])
        .unwrap_or_else(|e| {
            println!("{}", e);
            std::process::exit(1);
        });
    println!("{:#?}", macho);
}

fn usage(args: Vec<String>) {
    println!("usage: {} FILENAME", args[0]);
}

fn mmap_file(path: String) -> Result<Mmap, String> {
    let file = File::open(path)
        .map_err(|e| format!("error opening file: {}", e))?;
    unsafe {
        MmapOptions::new().map(&file)
            .map_err(|e| format!("error mmaping file: {}", e))
    }
}
