fn main() {
    if let Err(e) = zetaboy::run() {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
