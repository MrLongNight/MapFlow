#[cfg(windows)]
extern crate winres;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        #[cfg(windows)]
        copy_runtime_dlls();

        #[cfg(windows)]
        {
            let mut res = winres::WindowsResource::new();
            res.set_icon("../../resources/app_icons/Vorce_Logo_LQ-Full.ico");
            res.compile().unwrap();
        }
    }
}

#[cfg(windows)]
fn copy_runtime_dlls() {
    use std::collections::HashSet;
    use std::fs;

    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"),
    );
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Failed to resolve target profile dir")
        .to_path_buf();
    let workspace_root = manifest_dir.join("..").join("..");
    let runtime_dirs = [
        workspace_root
            .join("vcpkg_installed")
            .join("x64-windows")
            .join("bin"),
        workspace_root
            .join("vcpkg")
            .join("packages")
            .join("ffmpeg_x64-windows")
            .join("bin"),
        workspace_root
            .join("vcpkg")
            .join("installed")
            .join("x64-windows")
            .join("bin"),
    ];

    for runtime_dir in &runtime_dirs {
        println!("cargo:rerun-if-changed={}", runtime_dir.display());
    }

    let dll_prefixes = ["av", "sw", "pkgconf"];
    let mut copied_names = HashSet::new();
    let mut copied_any = false;

    for runtime_dir in runtime_dirs.iter().filter(|dir| dir.is_dir()) {
        let entries = match fs::read_dir(runtime_dir) {
            Ok(entries) => entries,
            Err(err) => {
                println!(
                    "cargo:warning=Failed to read runtime DLL directory {}: {}",
                    runtime_dir.display(),
                    err
                );
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    println!("cargo:warning=Failed to inspect runtime DLL entry: {err}");
                    continue;
                }
            };

            let path = entry.path();
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };
            let is_runtime_dll = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("dll"))
                .unwrap_or(false)
                && dll_prefixes
                    .iter()
                    .any(|prefix| file_name.starts_with(prefix));
            if !is_runtime_dll || copied_names.contains(file_name) {
                continue;
            }

            if !is_valid_runtime_dll(&path) {
                println!(
                    "cargo:warning=Skipping invalid runtime DLL candidate: {}",
                    path.display()
                );
                continue;
            }

            let destination = profile_dir.join(file_name);
            match fs::copy(&path, &destination) {
                Ok(_) => {
                    copied_names.insert(file_name.to_owned());
                    copied_any = true;
                }
                Err(err) => {
                    println!(
                        "cargo:warning=Failed to copy runtime DLL {} -> {}: {}",
                        path.display(),
                        destination.display(),
                        err
                    );
                }
            }
        }
    }

    if !copied_any {
        println!(
            "cargo:warning=No runtime DLLs were copied into {}",
            profile_dir.display()
        );
    }
}

#[cfg(windows)]
fn is_valid_runtime_dll(path: &std::path::Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name.starts_with("pkgconf") {
        return true;
    }

    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut dos_header = [0_u8; 64];
    if file.read_exact(&mut dos_header).is_err() || &dos_header[0..2] != b"MZ" {
        return false;
    }

    let pe_offset = u32::from_le_bytes([
        dos_header[0x3c],
        dos_header[0x3d],
        dos_header[0x3e],
        dos_header[0x3f],
    ]) as u64;

    if file.seek(SeekFrom::Start(pe_offset)).is_err() {
        return false;
    }

    let mut pe_signature = [0_u8; 4];
    file.read_exact(&mut pe_signature).is_ok() && pe_signature == *b"PE\0\0"
}
