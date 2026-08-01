//! Build script that helps `cargo build` and `cargo test` link against the
//! Python shared library when the `extension-module` feature is disabled.
//!
//! Some Linux distributions (notably the Ubuntu minimal image used here) ship
//! `libpython3.X.so.1.0` in `LIBDIR` but put the unversioned `libpython3.X.so`
//! symlink only under the `LIBPL` config directory. `pyo3-build-config` picks
//! up `LIBDIR` and `-lpython3.X`, so the linker cannot resolve the library
//! unless we add `LIBPL` (or an equivalent directory containing the `.so` link)
//! to the link search path.
//!
//! When `extension-module` is enabled (e.g. `maturin build`/`develop`), the
//! crate is built as a true extension module and must *not* link `libpython`,
//! so this script does nothing.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Only intervene for non-extension-module builds (cargo build / cargo test).
    if env::var("CARGO_FEATURE_EXTENSION_MODULE").is_ok() {
        return;
    }

    let python = env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let output = Command::new(&python)
        .args([
            "-c",
            "import sysconfig; \
             print(sysconfig.get_config_var('LDLIBRARY')); \
             print(sysconfig.get_config_var('LIBDIR')); \
             print(sysconfig.get_config_var('LIBPL')); \
             print(sysconfig.get_config_var('VERSION'))",
        ])
        .output()
        .expect("failed to run Python sysconfig query");

    if !output.status.success() {
        // If Python cannot be queried, fall back to pyo3's own configuration.
        return;
    }

    let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if lines.len() < 4 {
        return;
    }

    let ld_library = &lines[0];
    let lib_dir = &lines[1];
    let lib_pl = &lines[2];
    let _version = &lines[3];

    // If the requested unversioned library already exists in LIBDIR, nothing to do.
    let lib_dir_path = Path::new(lib_dir);
    if lib_dir_path.join(ld_library).is_file() {
        return;
    }

    // Otherwise, try to locate the actual shared object (e.g. libpython3.X.so.1.0)
    // in LIBPL or LIBDIR and create a local symlink in OUT_DIR for the linker.
    let candidates: Vec<&str> = vec![lib_pl, lib_dir];
    let actual_so = candidates
        .iter()
        .copied()
        .map(Path::new)
        .filter(|dir| dir.is_dir())
        .flat_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |name| name.starts_with(ld_library) && name != ld_library)
        });

    if let Some(actual) = actual_so {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        let link = out_dir.join(ld_library);
        if link.exists() {
            let _ = fs::remove_file(&link);
        }
        if std::os::unix::fs::symlink(&actual, &link).is_ok() {
            println!("cargo:rustc-link-search=native={}", out_dir.display());
        }
    }
}
