// Builtin samfile
pub mod buildin {
    include!(concat!(env!("OUT_DIR"), "/builtin_filtered.rs"));
}

pub mod license;
pub mod pack;
pub mod linksaver;


pub fn printbanner() {
    // https://www.asciiart.eu/text-to-ascii-art
    // DOS Rebel

    println!(r#"
  █████████  ██████████   █████████ 
 ███░░░░░███░░███░░░░░█  ███░░░░░███
░███    ░░░  ░███  █ ░  ███     ░░░ 
░░█████████  ░██████   ░███         
 ░░░░░░░░███ ░███░░█   ░███    █████
 ███    ░███ ░███ ░   █░░███  ░░███ 
░░█████████  ██████████ ░░█████████ 
 ░░░░░░░░░  ░░░░░░░░░░   ░░░░░░░░░  

The runner for samfiles
https://shadowdara.github.io/docs/#/samfile
"#);
}
