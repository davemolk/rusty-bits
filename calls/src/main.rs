fn main()  {
    if let Err(e) = calls::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
