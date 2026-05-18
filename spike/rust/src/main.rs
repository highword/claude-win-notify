fn main() {
    println!("RustSpike: Hello from Rust!");
    println!("  Version: {}", env!("CARGO_PKG_VERSION"));
    println!("  Target: {}", std::env::consts::ARCH);
    println!("  OS: {}", std::env::consts::OS);
}
