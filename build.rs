#[cfg(feature = "protoc-rust")]
use protobuf_codegen::Codegen;
#[cfg(feature = "protoc-rust")]
use std::{env, ffi::OsStr, fs, path::Path};

#[cfg(feature = "protoc-rust")]
fn proto_modules(proto_dir: &Path) -> Vec<String> {
    fs::read_dir(proto_dir)
        .expect("Could not read protobuf directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension() == Some(OsStr::new("proto")) {
                path.file_stem()
                    .and_then(|n| n.to_os_string().into_string().ok())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(feature = "protoc-rust")]
fn main() {
    let include_root = Path::new("s2client-proto");
    let include_sub = include_root.join("s2clientprotocol");
    let out_dir = &env::var("OUT_DIR").unwrap();

    if !include_sub.exists() {
        panic!(
            "Proto directory not found: {}. Did you fetch submodules? \
             Try: git submodule update --init --recursive",
            include_sub.display()
        );
    }
    let input_mods = proto_modules(&include_sub);

    // Inputs must be under an include path -> prefix with "s2client-proto/"
    let input_files_rel: Vec<String> = input_mods
        .iter()
        .map(|s| format!("s2client-proto/s2clientprotocol/{}.proto", s))
        .collect();
    let input_file_refs: Vec<&str> = input_files_rel.iter().map(|s| s.as_str()).collect();

    // Re-run build if any proto changes
    for s in &input_mods {
        let p = include_sub.join(format!("{s}.proto"));
        println!("cargo:rerun-if-changed={}", p.display());
    }
    println!("cargo:rerun-if-changed=s2client-proto");

    // Vendored protoc and well-known types include
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("Failed to locate vendored protoc");
    let protoc_include = protoc_bin_vendored::include_path().expect("Failed to get protoc include");

    // Include both roots to satisfy imports like "common.proto" and "s2clientprotocol/common.proto"
    let include_paths = vec![
        include_root.to_str().unwrap(),
        include_sub.to_str().unwrap(),
        protoc_include.to_str().unwrap(),
    ];

    Codegen::new()
        .protoc_path(&protoc)
        .includes(&include_paths)
        .inputs(&input_file_refs)
        .out_dir(out_dir)
        .run()
        .expect("protoc codegen failed");

    // Generate lib.rs listing modules
    fs::write(
        format!("{}/{}", out_dir, "lib.rs"),
        input_mods
            .iter()
            .map(|s| format!("pub mod {};", s))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}

#[cfg(not(feature = "protoc-rust"))]
fn main() {
    println!("using pre-generated *.rs files in 'src/'");
}
