fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).map(|s| s.as_str());
    
    if let Err(e) = zetaboy::run(rom_path) {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
