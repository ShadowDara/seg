// ============================================================
// PYTHON / VENV
// ============================================================

use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::license::{LICENSE_FILE_PATTERNS, safe_name,append_license_to_bundle};

fn find_python_site_packages() -> Vec<PathBuf> {
    let mut result = Vec::new();

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return result,
    };

    // Common virtual environment names.
    let venv_names = [
        ".venv",
        "venv",
        "env",
        ".env",
    ];

    for venv_name in venv_names {
        let venv = current_dir.join(venv_name);

        if !venv.is_dir() {
            continue;
        }

        // Linux / macOS:
        //
        // .venv/lib/python3.12/site-packages
        let lib_dir = venv.join("lib");

        if lib_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if !path.is_dir() {
                        continue;
                    }

                    let filename =
                        path.file_name()
                            .map(|x| x.to_string_lossy())
                            .unwrap_or_default();

                    if filename.starts_with("python") {
                        let site_packages =
                            path.join("site-packages");

                        if site_packages.is_dir() {
                            result.push(site_packages);
                        }
                    }
                }
            }
        }

        // Windows:
        //
        // .venv/Lib/site-packages
        let windows_site_packages =
            venv.join("Lib").join("site-packages");

        if windows_site_packages.is_dir() {
            result.push(windows_site_packages);
        }
    }

    result.sort();
    result.dedup();

    result
}


fn normalize_python_package_name(
    name: &str,
) -> String {
    name.replace('_', "-")
        .replace('.', "-")
}


fn parse_python_metadata(
    metadata: &Path,
) -> (String, String, String) {
    let content =
        match fs::read_to_string(metadata) {
            Ok(content) => content,
            Err(_) => {
                return (
                    String::new(),
                    String::new(),
                    "UNKNOWN".to_string(),
                );
            }
        };

    let mut name = String::new();
    let mut version = String::new();
    let mut license = String::new();

    for line in content.lines() {
        if let Some(value) =
            line.strip_prefix("Name:")
        {
            name = value.trim().to_string();
        }

        if let Some(value) =
            line.strip_prefix("Version:")
        {
            version = value.trim().to_string();
        }

        if let Some(value) =
            line.strip_prefix("License:")
        {
            license = value.trim().to_string();
        }

        if !name.is_empty()
            && !version.is_empty()
            && !license.is_empty()
        {
            break;
        }
    }

    if license.is_empty() {
        license = "UNKNOWN".to_string();
    }

    (name, version, license)
}


fn find_python_license_files(
    dist_info_dir: &Path,
) -> Vec<PathBuf> {
    let mut result = Vec::new();

    // PEP 639 / modern wheel layout:
    //
    // package-x.y.z.dist-info/licenses/LICENSE
    let licenses_dir =
        dist_info_dir.join("licenses");

    if licenses_dir.is_dir() {
        collect_license_files_recursive(
            &licenses_dir,
            &mut result,
        );
    }

    // Older packages sometimes put LICENSE directly
    // into the .dist-info directory.
    for filename in LICENSE_FILE_PATTERNS {
        let path =
            dist_info_dir.join(filename);

        if path.is_file()
            && !result.contains(&path)
        {
            result.push(path);
        }
    }

    // Some packages have files such as:
    //
    // LICENSE-MIT
    // LICENSE-APACHE
    // COPYING
    //
    if let Ok(entries) =
        fs::read_dir(dist_info_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(filename) =
                path.file_name()
            else {
                continue;
            };

            let filename =
                filename
                    .to_string_lossy()
                    .to_uppercase();

            if filename.starts_with("LICENSE")
                || filename.starts_with("LICENCE")
                || filename.starts_with("COPYING")
            {
                if !result.contains(&path) {
                    result.push(path);
                }
            }
        }
    }

    result.sort();
    result
}


fn collect_license_files_recursive(
    directory: &Path,
    result: &mut Vec<PathBuf>,
) {
    let entries =
        match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return,
        };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_license_files_recursive(
                &path,
                result,
            );

            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Some(filename) =
            path.file_name()
        else {
            continue;
        };

        let filename =
            filename
                .to_string_lossy()
                .to_uppercase();

        if filename.starts_with("LICENSE")
            || filename.starts_with("LICENCE")
            || filename.starts_with("COPYING")
        {
            result.push(path);
        }
    }
}


fn copy_python_license_files(
    dist_info_dir: &Path,
    destination_root: &Path,
    name: &str,
    version: &str,
) -> Vec<PathBuf> {
    let license_files =
        find_python_license_files(
            dist_info_dir,
        );

    if license_files.is_empty() {
        return Vec::new();
    }

    let destination_dir =
        destination_root
            .join(safe_name(name))
            .join(version);

    if let Err(err) =
        fs::create_dir_all(
            &destination_dir,
        )
    {
        println!(
            "  WARNING: Could not create {}: {}",
            destination_dir.display(),
            err
        );

        return Vec::new();
    }

    let mut copied =
        Vec::new();

    for license_file in license_files {
        let Some(filename) =
            license_file.file_name()
        else {
            continue;
        };

        let destination =
            destination_dir.join(filename);

        match fs::copy(
            &license_file,
            &destination,
        ) {
            Ok(_) => {
                copied.push(destination);
            }

            Err(err) => {
                println!(
                    "  WARNING: Could not copy {}: {}",
                    license_file.display(),
                    err
                );
            }
        }
    }

    copied
}


pub fn process_python(
    bundle: &mut File, verbose: bool
) -> usize {
    println!();
    println!("{}", "=".repeat(80));
    println!("PYTHON PACKAGES");
    println!("{}", "=".repeat(80));

    let site_packages =
        find_python_site_packages();

    if site_packages.is_empty() {
        println!(
            "No Python virtual environment found."
        );

        println!(
            "Expected one of: .venv, venv, env, .env"
        );

        return 0;
    }

    let python_output =
        PathBuf::from("third_party_licenses")
            .join("python");

    if let Err(err) =
        fs::create_dir_all(
            &python_output,
        )
    {
        println!(
            "Could not create Python output directory: {}",
            err
        );

        return 0;
    }

    let mut count = 0usize;
    let mut license_count = 0usize;

    let mut processed =
        HashSet::new();

    for site_package in site_packages {
        let entries =
            match fs::read_dir(
                &site_package,
            ) {
                Ok(entries) => entries,

                Err(err) => {
                    println!(
                        "Could not read {}: {}",
                        site_package.display(),
                        err
                    );

                    continue;
                }
            };

        for entry in entries.flatten() {
            let dist_info =
                entry.path();

            if !dist_info.is_dir() {
                continue;
            }

            let Some(dirname) =
                dist_info.file_name()
            else {
                continue;
            };

            let dirname =
                dirname.to_string_lossy();

            if !dirname.ends_with(".dist-info") {
                continue;
            }

            let metadata =
                dist_info.join("METADATA");

            if !metadata.is_file() {
                continue;
            }

            let (
                name,
                version,
                license_name,
            ) =
                parse_python_metadata(
                    &metadata,
                );

            if name.is_empty()
                || version.is_empty()
            {
                continue;
            }

            let identifier =
                format!(
                    "{}@{}",
                    normalize_python_package_name(
                        &name
                    ),
                    version
                );

            if !processed.insert(identifier) {
                continue;
            }

            let license_files =
                copy_python_license_files(
                    &dist_info,
                    &python_output,
                    &name,
                    &version,
                );

            let source_url =
                format!(
                    "https://pypi.org/project/{}/",
                    name
                );

            if let Err(err) =
                append_license_to_bundle(
                    bundle,
                    "python",
                    &name,
                    &version,
                    &license_name,
                    &license_files,
                    &source_url,
                )
            {
                println!(
                    "  WARNING: Could not write bundle: {}",
                    err
                );
            }

            count += 1;
            license_count +=
                license_files.len();

            if !license_files.is_empty() {
                if verbose {
                println!(
                    "[OK] {}@{} ({} license file(s))",
                    name,
                    version,
                    license_files.len()
                );}
            } else {
                println!(
                    "[WARNING] {}@{} - no license file found",
                    name,
                    version
                );
            }
        }
    }

    println!();
    println!(
        "Python packages: {}",
        count
    );

    println!(
        "Python license files: {}",
        license_count
    );

    count
}
