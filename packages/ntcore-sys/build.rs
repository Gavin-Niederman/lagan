fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-search=native={manifest_dir}/link");
    println!("cargo:rustc-link-lib=ntcore");
    println!("cargo:rustc-link-lib=wpiutil");
    println!("cargo:rustc-link-lib=wpinet");
}
