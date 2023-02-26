fn main() {
    // Rebuild if memory.x file was rebuild.
    // Linker seems to pick it up automatically.
    // (via the include in link.x which is added with a linker argument!)
    println!("cargo:rerun-if-changed=memory.x");
}
