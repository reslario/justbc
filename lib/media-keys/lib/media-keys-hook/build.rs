fn main() {
    // this makes the ".shared" section actually shared,
    // meaning every instance of the dll will access
    // the same memory
    // (RWS = read/write/shared)
    #[cfg(windows)]
    println!(r"cargo:rustc-cdylib-link-arg=/section:.shared,RWS")
}
