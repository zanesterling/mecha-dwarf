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
}

fn usage(args: Vec<String>) {
    println!("usage: {} FILENAME", args[0]);
}
