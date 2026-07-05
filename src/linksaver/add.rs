use std::path::Path;

use chrono::Utc;

use crate::linksaver::{AppConfig, Link4, prompt, save};

// add4
// replace add2
pub fn add4(c: &mut AppConfig) {
    let entry = prompt("Entry text: ");
    let l = Link4 {
        link: entry,
        date: Utc::now().to_rfc3339(),
    };
    c.links4.push(l);
    let _ = save(c);
    println!("Added new entry!");
}

// add5#
// replace add3
pub fn add5(c: &mut AppConfig) {
    let path = prompt("License file: ");

    if !Path::new(&path).exists() {
        println!("Warning: '{}' does not exist.", path);
    }

    let l = Link4 {
        link: path,
        date: Utc::now().to_rfc3339(),
    };

    c.links5.push(l);

    let _ = save(c);
    println!("Added license file!");
}
