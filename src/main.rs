fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--check") {
        println!("check: ok (nothing to check yet)");
    }
}
