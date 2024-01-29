use std::fs::File;

use memmap::{ Mmap, MmapOptions };

mod dwarf;
mod leb;
mod macho;

struct Config {
    path: String,
    verbose: bool,
}

fn main() {
    let config = parse_config(std::env::args());
    let mmap: Mmap = mmap_file(config.path)
        .unwrap_or_else(|e| {
            println!("{}", e);
            std::process::exit(1);
        });

    // Parse the Mach-O file.
    let macho = macho::File::from(&mmap[..])
        .unwrap_or_else(|e| {
            println!("error parsing macho: {}", e);
            std::process::exit(1);
        });
    if config.verbose {
        println!("{:#x?}", macho);
    }

    // Get the DWARF segment
    let dwarf_seg = macho.load_commands.into_iter()
        .filter_map(|cmd| {
            if let macho::LoadCommandDetails::Segment64(seg) = cmd.details {
                if seg.segname.as_str() == "__DWARF" { return Some(seg); }
            }
            None
        })
        .next()
        .unwrap_or_else(|| {
            println!("error: file has no __DWARF segment");
            std::process::exit(1);
        });
    if config.verbose {
        println!("{:#x?}", dwarf_seg);
    }

    let dwarf_file = dwarf::File::from(dwarf_seg, &mmap)
        .unwrap_or_else(|e| {
            println!("error parsing dwarf: {}", e);
            std::process::exit(1);
        });
    println!("{}", dwarf_file);
}

fn usage(args: Vec<String>) {
    println!("usage: {} [-v] FILENAME", args[0]);
}

fn parse_config(args: std::env::Args) -> Config {
    let mut args: Vec<String> = args.collect();
    let mut config = Config {
        path: String::from(""),
        verbose: false,
    };
    for i in 1..args.len() {
        if args[i] == "-v" {
            config.verbose = true;
            args.swap_remove(i);
            break;
        }
    }
    if args.len() != 2 {
        usage(args);
        std::process::exit(1);
    }
    config.path = args.swap_remove(1);
    config
}

fn mmap_file(path: String) -> Result<Mmap, String> {
    let file = File::open(path)
        .map_err(|e| format!("error opening file: {}", e))?;
    unsafe {
        MmapOptions::new().map(&file)
            .map_err(|e| format!("error mmaping file: {}", e))
    }
}
