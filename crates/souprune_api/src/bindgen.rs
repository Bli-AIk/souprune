//! Binding generator for souprune API.
//! Generates C and C# bindings from the FFI types defined in this crate.
//!
//! 绑定生成器。
//! 从此 crate 中定义的 FFI 类型生成 C 和 C# 绑定。

use interoptopus::inventory::Inventory;
use souprune_api::bindgen_inventory::build_inventory;
use std::path::Path;

fn main() {
    let inventory = build_inventory();

    // Determine output directory
    let out_dir = std::env::var("SOUPRUNE_BINDGEN_OUT").unwrap_or_else(|_| "generated".to_string());
    let out_path = Path::new(&out_dir);

    std::fs::create_dir_all(out_path).expect("Failed to create output directory");

    // Generate C header
    generate_c_bindings(&inventory, out_path);

    // Generate C# bindings
    generate_csharp_bindings(&inventory, out_path);

    println!("Bindings generated successfully in: {}", out_path.display());
}

fn generate_c_bindings(inventory: &Inventory, out_path: &Path) {
    use interoptopus_backend_c::Interop;

    let c_path = out_path.join("souprune_api.h");

    Interop::builder()
        .inventory(inventory.clone())
        .build()
        .expect("Failed to build C interop")
        .write_file(&c_path)
        .expect("Failed to write C header");

    println!("Generated C header: {}", c_path.display());
}

fn generate_csharp_bindings(inventory: &Inventory, out_path: &Path) {
    use interoptopus_backend_csharp::Interop;

    let csharp_path = out_path.join("SoupruneApi.cs");

    Interop::builder()
        .inventory(inventory.clone())
        .build()
        .expect("Failed to build C# interop")
        .write_file(&csharp_path)
        .expect("Failed to write C# bindings");

    println!("Generated C# bindings: {}", csharp_path.display());
}
