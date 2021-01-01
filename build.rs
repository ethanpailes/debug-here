fn main() -> Result<(), Box<dyn std::error::Error>> {
    let triple = std::env::var("TARGET").unwrap();
    if triple.contains("macabi") {
        println!("cargo:rustc-cfg=mac_catalyst");
    }

    Ok(())
}
