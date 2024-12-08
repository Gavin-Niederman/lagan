use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let release = std::env::var("PROFILE").unwrap() == "release";
    let maybe_d = if release { "" } else { "d" };

    cmake::Config::new(format!("{manifest_dir}/allwpilib"))
        .define("WITH_CSCORE", "OFF")
        .define("WITH_GUI", "OFF")
        .define("WITH_SIMULATION_MODULES", "OFF")
        .define("WITH_TESTS", "OFF")
        .define("WITH_WPILIB", "OFF")
        .define("WITH_WPIMATH", "OFF")
        .define("WITH_PROTOBUF", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    Command::new("cmake")
        .args(["--build", &out_dir, "-j", "8"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-search=native={out_dir}/lib");
    println!("cargo:rustc-link-lib=ntcore{maybe_d}");
    println!("cargo:rustc-link-lib=wpiutil{maybe_d}");
    println!("cargo:rustc-link-lib=wpinet{maybe_d}");
}
