use std::fs::File;
use std::io::BufWriter;

use ico::{IconDir, ResourceType};

pub fn genicon(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[INFO] Starting ICO generation for: {}", name);

    let mut icon_dir = IconDir::new(ResourceType::Icon);

    let sizes = [16, 32, 48, 64, 128, 256];

    println!("[INFO] Target sizes: {:?}", sizes);

    for size in sizes {
        println!("\n[DEBUG] Processing size: {}x{}", size, size);

        // Load image
        println!("[DEBUG] Loading image...");
        let img = match image::open(name) {
            Ok(img) => {
                println!("[DEBUG] Image loaded successfully");
                img
            }
            Err(e) => {
                println!("[ERROR] Failed to load image: {}", e);
                return Err(Box::new(e));
            }
        };

        // Resize
        println!("[DEBUG] Resizing...");
        let resized = img
            .resize(size, size, image::imageops::FilterType::Lanczos3)
            .to_rgba8();

        println!(
            "[DEBUG] Resize done. Dimensions: {}x{}",
            resized.width(),
            resized.height()
        );

        // Encode ICO image
        println!("[DEBUG] Encoding ICO image...");
        let icon_image =
            ico::IconImage::from_rgba_data(size, size, resized.clone().into_raw());

        let entry = match ico::IconDirEntry::encode(&icon_image) {
            Ok(e) => {
                println!("[DEBUG] Encoding successful for {}x{}", size, size);
                e
            }
            Err(e) => {
                println!("[ERROR] Encoding failed for {}x{}: {}", size, size, e);
                return Err(Box::new(e));
            }
        };

        icon_dir.add_entry(entry);
        println!("[DEBUG] Entry added to IconDir");
    }

    println!("\n[INFO] Writing ICO file...");

    let out_path = format!("{}.ico", name);

    let file = match File::create(&out_path) {
        Ok(f) => {
            println!("[DEBUG] Output file created: {}", out_path);
            f
        }
        Err(e) => {
            println!("[ERROR] Failed to create output file: {}", e);
            return Err(Box::new(e));
        }
    };

    let writer = BufWriter::new(file);

    match icon_dir.write(writer) {
        Ok(_) => println!("[SUCCESS] ICO written successfully: {}", out_path),
        Err(e) => {
            println!("[ERROR] Failed to write ICO: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}
