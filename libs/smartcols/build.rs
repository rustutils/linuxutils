use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.first().unwrap_or(&"0");
    let minor = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");

    // Read the .h.in template and substitute version placeholders.
    let template =
        fs::read_to_string(manifest_dir.join("include/libsmartcols.h.in"))
            .unwrap();
    let header = template
        .replace("@LIBSMARTCOLS_VERSION@", &version)
        .replace("@LIBSMARTCOLS_MAJOR_VERSION@", major)
        .replace("@LIBSMARTCOLS_MINOR_VERSION@", minor)
        .replace("@LIBSMARTCOLS_PATCH_VERSION@", patch);

    let generated_header = out_dir.join("libsmartcols.h");
    fs::write(&generated_header, &header).unwrap();

    // Run bindgen on the generated header to extract constants.
    let bindings = bindgen::Builder::default()
        .header(generated_header.to_str().unwrap())
        .allowlist_var("SCOLS_.*")
        .allowlist_var("LIBSMARTCOLS_.*")
        .allowlist_type("libscols_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings");

    println!("cargo::rerun-if-changed=include/libsmartcols.h.in");
}
