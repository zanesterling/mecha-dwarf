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

    println!("loading from file: {}", path);
    let mut bytes: [u8; 32] = [0; 32];
    bytes[0] = 0xcf;
    bytes[1] = 0xfa;
    bytes[2] = 0xed;
    bytes[3] = 0xfe;
    let header = macho::Header::from(bytes);
    header.show()
}

fn usage(args: Vec<String>) {
    println!("usage: {} FILENAME", args[0]);
}
