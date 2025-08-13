fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom>", args[0]);
        std::process::exit(1);
    }
    let rom_path = &args[1];
    if let Err(e) = zetaboy::run(rom_path) {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
