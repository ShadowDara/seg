use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod python;
mod config;
mod npm;
mod cargo;

const LICENSE_FILE_PATTERNS: &[&str] = &[
    "LICENSE",
    "LICENSE.txt",
    "LICENSE.md",
    "LICENCE",
    "LICENCE.txt",
    "LICENCE.md",
    "COPYING",
    "COPYING.txt",
    "COPYING.md",
];


// ============================================================
// Utility
// ============================================================

fn safe_name(name: &str) -> String {
    name.replace('/', "__")
        .replace('\\', "__")
}


fn find_license_files(package_dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();

    if !package_dir.is_dir() {
        return found;
    }

    // Exact/common filenames
    for filename in LICENSE_FILE_PATTERNS {
        let path = package_dir.join(filename);

        if path.is_file() {
            found.push(path);
        }
    }

    // Variants such as:
    //
    // LICENSE-MIT
    // LICENSE-APACHE
    // LICENSE-MIT.txt
    // LICENCE-MIT
    // COPYING-LGPL

    if let Ok(entries) = fs::read_dir(package_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(filename) = path.file_name() else {
                continue;
            };

            let filename = filename.to_string_lossy().to_uppercase();

            if filename.starts_with("LICENSE")
                || filename.starts_with("LICENCE")
                || filename.starts_with("COPYING")
            {
                if !found.contains(&path) {
                    found.push(path);
                }
            }
        }
    }

    found.sort();

    found
}


fn copy_license_files(
    source_dir: &Path,
    destination_root: &Path,
    name: &str,
    version: &str,
) -> Vec<PathBuf> {
    let license_files = find_license_files(source_dir);

    if license_files.is_empty() {
        return Vec::new();
    }

    let destination_dir = destination_root
        .join(safe_name(name))
        .join(version);

    if let Err(err) = fs::create_dir_all(&destination_dir) {
        println!(
            "  WARNING: Could not create {}: {}",
            destination_dir.display(),
            err
        );

        return Vec::new();
    }

    let mut copied = Vec::new();

    for license_file in license_files {
        let Some(filename) = license_file.file_name() else {
            continue;
        };

        let destination = destination_dir.join(filename);

        match fs::copy(&license_file, &destination) {
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


fn append_license_to_bundle(
    bundle: &mut File,
    ecosystem: &str,
    name: &str,
    version: &str,
    license_name: &str,
    license_files: &[PathBuf],
    source_url: &str,
) -> io::Result<()> {
    writeln!(bundle)?;
    writeln!(bundle, "{}", "=".repeat(80))?;
    writeln!(
        bundle,
        "{}: {}@{}",
        ecosystem.to_uppercase(),
        name,
        version
    )?;
    writeln!(bundle, "{}", "=".repeat(80))?;
    writeln!(bundle)?;

    writeln!(
        bundle,
        "License metadata: {}",
        license_name
    )?;

    writeln!(
        bundle,
        "Source: {}",
        source_url
    )?;

    writeln!(bundle)?;

    if license_files.is_empty() {
        writeln!(
            bundle,
            "NO LICENSE FILE FOUND IN PACKAGE"
        )?;

        writeln!(bundle)?;

        return Ok(());
    }

    for license_file in license_files {
        let filename = license_file
            .file_name()
            .map(|x| x.to_string_lossy())
            .unwrap_or_default();

        writeln!(
            bundle,
            "--- {} ---",
            filename
        )?;

        writeln!(bundle)?;

        match fs::read_to_string(license_file) {
            Ok(content) => {
                write!(bundle, "{}", content)?;

                if !content.ends_with('\n') {
                    writeln!(bundle)?;
                }

                writeln!(bundle)?;
            }

            Err(err) => {
                writeln!(
                    bundle,
                    "Could not read license file: {}",
                    err
                )?;

                writeln!(bundle)?;
            }
        }
    }

    Ok(())
}


// ============================================================
// NPM
// ============================================================

fn get_npm_license(package_json: &Path) -> String {
    let content = match fs::read_to_string(package_json) {
        Ok(content) => content,
        Err(_) => return "UNKNOWN".to_string(),
    };

    let package: Value = match serde_json::from_str(&content) {
        Ok(package) => package,
        Err(_) => return "UNKNOWN".to_string(),
    };

    let license_value = package.get("license");

    // Modern:
    //
    // "license": "MIT"

    if let Some(license) = license_value {
        if let Some(value) = license.as_str() {
            return value.to_string();
        }

        // Older:
        //
        // "license": {
        //     "type": "MIT"
        // }

        if let Some(object) = license.as_object() {
            if let Some(value) =
                object.get("type").and_then(|x| x.as_str())
            {
                return value.to_string();
            }
        }
    }

    // Very old:
    //
    // "licenses": [
    //     {
    //         "type": "MIT"
    //     }
    // ]

    if let Some(licenses) = package
        .get("licenses")
        .and_then(|x| x.as_array())
    {
        let mut values = Vec::new();

        for item in licenses {
            if let Some(value) = item.as_str() {
                values.push(value.to_string());
            } else if let Some(object) = item.as_object() {
                if let Some(value) =
                    object.get("type").and_then(|x| x.as_str())
                {
                    values.push(value.to_string());
                }
            }
        }

        if !values.is_empty() {
            return values.join(", ");
        }
    }

    "UNKNOWN".to_string()
}


fn process_npm(bundle: &mut File, verbose: bool) -> usize {
    let root = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            println!(
                "Could not determine current directory: {}",
                err
            );

            return 0;
        }
    };

    let lock_file = root.join("package-lock.json");
    let node_modules = root.join("node_modules");

    if !lock_file.exists() {
        println!("package-lock.json not found.");
        return 0;
    }

    if !node_modules.exists() {
        println!("node_modules not found.");
        println!("Run 'npm install' first.");
        return 0;
    }

    println!();
    println!("{}", "=".repeat(80));
    println!("NPM PACKAGES");
    println!("{}", "=".repeat(80));

    let content = match fs::read_to_string(&lock_file) {
        Ok(content) => content,
        Err(err) => {
            println!(
                "Could not read package-lock.json: {}",
                err
            );

            return 0;
        }
    };

    let lock: Value = match serde_json::from_str(&content) {
        Ok(lock) => lock,
        Err(err) => {
            println!(
                "Could not parse package-lock.json: {}",
                err
            );

            return 0;
        }
    };

    let packages = match lock
        .get("packages")
        .and_then(|x| x.as_object())
    {
        Some(packages) => packages,
        None => {
            println!(
                "No 'packages' section found in package-lock.json."
            );

            return 0;
        }
    };

    let npm_output = PathBuf::from("third_party_licenses")
        .join("npm");

    if let Err(err) = fs::create_dir_all(&npm_output) {
        println!(
            "Could not create npm output directory: {}",
            err
        );

        return 0;
    }

    let mut count = 0usize;
    let mut license_count = 0usize;

    for (key, value) in packages {
        // Root package
        if key.is_empty() {
            continue;
        }

        let package_path = root.join(key);

        if !package_path.is_dir() {
            continue;
        }

        let package_json = package_path.join("package.json");

        if !package_json.exists() {
            continue;
        }

        let name = value
            .get("name")
            .and_then(|x| x.as_str())
            .map(|x| x.to_string())
            .unwrap_or_else(|| {
                key.strip_prefix("node_modules/")
                    .unwrap_or(key)
                    .to_string()
            });

        let version = match value
            .get("version")
            .and_then(|x| x.as_str())
        {
            Some(version) if !version.is_empty() => {
                version.to_string()
            }

            _ => continue,
        };

        let license_name =
            get_npm_license(&package_json);

        let license_files = copy_license_files(
            &package_path,
            &npm_output,
            &name,
            &version,
        );

        let source_url =
            format!(
                "https://www.npmjs.com/package/{}",
                name
            );

        if let Err(err) = append_license_to_bundle(
            bundle,
            "npm",
            &name,
            &version,
            &license_name,
            &license_files,
            &source_url,
        ) {
            println!(
                "  WARNING: Could not write bundle: {}",
                err
            );
        }

        count += 1;

        if !license_files.is_empty() {
            license_count += license_files.len();

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

    println!();
    println!("NPM packages: {}", count);
    println!(
        "NPM license files: {}",
        license_count
    );

    count
}


// ============================================================
// CARGO
// ============================================================

fn find_cargo_toml(
    cargo_registry: &Path,
    name: &str,
    version: &str,
) -> Option<PathBuf> {
    if !cargo_registry.exists() {
        return None;
    }

    let entries = fs::read_dir(cargo_registry).ok()?;

    for entry in entries.flatten() {
        let registry = entry.path();

        if !registry.is_dir() {
            continue;
        }

        let crate_dir = registry.join(
            format!("{}-{}", name, version)
        );

        let cargo_toml =
            crate_dir.join("Cargo.toml");

        if cargo_toml.exists() {
            return Some(cargo_toml);
        }
    }

    None
}


fn get_cargo_license(
    cargo_toml: &Path,
) -> String {
    let content = match fs::read_to_string(cargo_toml) {
        Ok(content) => content,
        Err(_) => return "UNKNOWN".to_string(),
    };

    // license = "MIT"

    if let Some(value) =
        extract_toml_string(&content, "license")
    {
        return value;
    }

    // license-file = "LICENSE"

    if let Some(value) =
        extract_toml_string(&content, "license-file")
    {
        return format!(
            "SEE LICENSE FILE ({})",
            value
        );
    }

    "UNKNOWN".to_string()
}


fn extract_toml_string(
    content: &str,
    key: &str,
) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();

        if !line.starts_with(key) {
            continue;
        }

        let rest = line[key.len()..].trim_start();

        if !rest.starts_with('=') {
            continue;
        }

        let value = rest[1..].trim();

        if value.len() >= 2
            && value.starts_with('"')
            && value.ends_with('"')
        {
            return Some(
                value[1..value.len() - 1]
                    .to_string()
            );
        }
    }

    None
}


fn find_cargo_license_files(
    crate_dir: &Path,
    cargo_toml: &Path,
) -> Vec<PathBuf> {
    let mut result =
        find_license_files(crate_dir);

    let content = match fs::read_to_string(cargo_toml) {
        Ok(content) => content,
        Err(_) => return result,
    };

    if let Some(license_file) =
        extract_toml_string(
            &content,
            "license-file",
        )
    {
        let path =
            crate_dir.join(license_file);

        if path.is_file()
            && !result.contains(&path)
        {
            result.push(path);
        }
    }

    result.sort();

    result
}


fn process_cargo(bundle: &mut File, verbose: bool) -> usize {
    let root = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            println!(
                "Could not determine current directory: {}",
                err
            );

            return 0;
        }
    };

    let lock_file =
        root.join("Cargo.lock");

    if !lock_file.exists() {
        println!("Cargo.lock not found.");
        return 0;
    }

    let home = match env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
    {
        Ok(home) => PathBuf::from(home),

        Err(_) => {
            println!(
                "Could not determine home directory."
            );

            return 0;
        }
    };

    let cargo_registry = home
        .join(".cargo")
        .join("registry")
        .join("src");

    if !cargo_registry.exists() {
        println!(
            "Cargo registry not found."
        );

        println!(
            "Run 'cargo fetch' first."
        );

        return 0;
    }

    println!();
    println!("{}", "=".repeat(80));
    println!("CARGO CRATES");
    println!("{}", "=".repeat(80));

    let lock = match fs::read_to_string(&lock_file) {
        Ok(content) => content,

        Err(err) => {
            println!(
                "Could not read Cargo.lock: {}",
                err
            );

            return 0;
        }
    };

    let cargo_output =
        PathBuf::from("third_party_licenses")
            .join("cargo");

    if let Err(err) =
        fs::create_dir_all(&cargo_output)
    {
        println!(
            "Could not create cargo output directory: {}",
            err
        );

        return 0;
    }

    let blocks: Vec<&str> =
        lock.split("[[package]]")
            .collect();

    let mut seen = HashSet::new();

    let mut count = 0usize;
    let mut license_count = 0usize;

    for block in blocks {
        let name =
            extract_lock_value(block, "name");

        let version =
            extract_lock_value(block, "version");

        let (Some(name), Some(version)) =
            (name, version)
        else {
            continue;
        };

        let identifier =
            format!("{}@{}", name, version);

        if seen.contains(&identifier) {
            continue;
        }

        seen.insert(identifier);

        let cargo_toml =
            find_cargo_toml(
                &cargo_registry,
                &name,
                &version,
            );

        let Some(cargo_toml) = cargo_toml else {
            println!(
                "[WARNING] {}@{} - Cargo source not found",
                name,
                version
            );

            let source_url =
                format!(
                    "https://crates.io/crates/{}",
                    name
                );

            if let Err(err) =
                append_license_to_bundle(
                    bundle,
                    "cargo",
                    &name,
                    &version,
                    "UNKNOWN",
                    &[],
                    &source_url,
                )
            {
                println!(
                    "  WARNING: Could not write bundle: {}",
                    err
                );
            }

            count += 1;
            continue;
        };

        let crate_dir =
            match cargo_toml.parent() {
                Some(path) => path,
                None => continue,
            };

        let license_name =
            get_cargo_license(&cargo_toml);

        let license_files =
            find_cargo_license_files(
                crate_dir,
                &cargo_toml,
            );

        let destination_dir =
            cargo_output
                .join(safe_name(&name))
                .join(&version);

        let mut copied_files = Vec::new();

        if !license_files.is_empty() {
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
            } else {
                for license_file
                    in license_files
                {
                    let Some(filename) =
                        license_file.file_name()
                    else {
                        continue;
                    };

                    let destination =
                        destination_dir
                            .join(filename);

                    match fs::copy(
                        &license_file,
                        &destination,
                    ) {
                        Ok(_) => {
                            copied_files.push(
                                destination
                            );
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
            }
        }

        let source_url =
            format!(
                "https://crates.io/crates/{}",
                name
            );

        if let Err(err) =
            append_license_to_bundle(
                bundle,
                "cargo",
                &name,
                &version,
                &license_name,
                &copied_files,
                &source_url,
            )
        {
            println!(
                "  WARNING: Could not write bundle: {}",
                err
            );
        }

        count += 1;
        license_count += copied_files.len();

        if !copied_files.is_empty() {
            if verbose {
            println!(
                "[OK] {}@{} ({} license file(s))",
                name,
                version,
                copied_files.len()
            );}
            
        } else {
            println!(
                "[WARNING] {}@{} - no license file found",
                name,
                version
            );
        }
    }

    println!();
    println!("Cargo crates: {}", count);
    println!(
        "Cargo license files: {}",
        license_count
    );

    count
}

fn extract_lock_value(
    block: &str,
    key: &str,
) -> Option<String> {
    for line in block.lines() {
        let line = line.trim();

        let prefix =
            format!("{} =", key);

        if !line.starts_with(&prefix) {
            continue;
        }

        let value = line[prefix.len()..]
            .trim();

        if value.len() >= 2
            && value.starts_with('"')
            && value.ends_with('"')
        {
            return Some(
                value[1..value.len() - 1]
                    .to_string()
            );
        }
    }

    None
}


// ============================================================
// Main
// ============================================================

pub fn lmain(verbose: bool, path: &str) {
    match config::load_config(path) {
        Ok(config) => {
            println!("Config erfolgreich geladen: {config:#?}");
            for entry in config {
               /* match entry {
                    entry::String => {
                        println!();
                    }
                }*/
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            return;
        }
    }
    
    println!("{}", "=".repeat(80));
    println!("THIRD-PARTY LICENSE BUNDLE");
    println!("{}", "=".repeat(80));

    let output_dir =
        PathBuf::from("third_party_licenses");

    let bundle_file =
        output_dir.join("LICENSE_BUNDLE.txt");

    if let Err(err) =
        fs::create_dir_all(&output_dir)
    {
        println!(
            "Could not create output directory: {}",
            err
        );

        return;
    }

    // Always create a fresh bundle.

    let mut bundle =
        match File::create(&bundle_file) {
            Ok(file) => file,

            Err(err) => {
                println!(
                    "Could not create license bundle: {}",
                    err
                );

                return;
            }
        };

    let now =
        chrono::Local::now()
            .to_rfc3339();

    if let Err(err) = writeln!(
        bundle,
        "THIRD-PARTY LICENSE BUNDLE"
    ) {
        println!(
            "Could not write bundle: {}",
            err
        );

        return;
    }

    let _ = writeln!(
        bundle,
        "Automatically generated from installed npm packages and Cargo crates."
    );

    let _ = writeln!(
        bundle,
        "Generated: {}",
        now
    );

    let _ = writeln!(bundle);

    let npm_count =
        process_npm(&mut bundle, verbose);

    let cargo_count =
        process_cargo(&mut bundle, verbose);
    
    let python_count = python::process_python(&mut bundle, verbose);

    println!();
    println!("{}", "=".repeat(80));
    println!("DONE");
    println!("{}", "=".repeat(80));
    println!();

    println!(
        "NPM packages : {}",
        npm_count
    );

    println!(
        "Cargo crates : {}",
        cargo_count
    );
    
    println!(
        "pip Packages : {}",
        python_count
    );

    println!();

    println!("License bundle:");
    println!(
        "  {}",
        bundle_file.display()
    );

    println!();

    println!("Copied license files:");
    println!(
        "  {}",
        output_dir.display()
    );
}
