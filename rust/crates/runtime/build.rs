//! Generates the compile-time index for embedded login portal assets.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read embedded portal directory") {
        let path = entry.expect("read embedded portal entry").path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"))
        .join("generated/portal");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    files.sort();
    let entries = files
        .iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&root)
                .expect("portal path below root")
                .to_string_lossy()
                .replace('\\', "/");
            format!(
                "({relative:?}, include_bytes!({path:?}).as_slice()),",
                path = path.canonicalize().expect("canonical portal path")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"))
        .join("portal_assets.rs");
    fs::write(output, format!("&[{entries}]\n")).expect("write embedded portal index");
}
