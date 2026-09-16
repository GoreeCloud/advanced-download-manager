#![forbid(unsafe_code)]

use goreecloud_download_core::PRODUCT_VERSION;

fn main() {
    let command = std::env::args().nth(1);
    match command.as_deref() {
        Some("--version" | "version") => {
            println!("gcdm {PRODUCT_VERSION} (Development source foundation)");
        }
        Some("status") => {
            println!("GoreeCloud Advanced Download Manager {PRODUCT_VERSION}");
            println!("Lifecycle: Development");
            println!("Download commands are not implemented yet.");
        }
        Some(_) => {
            eprintln!(
                "This Development source foundation does not implement download commands yet."
            );
            std::process::exit(2);
        }
        None => {
            println!("GoreeCloud Advanced Download Manager {PRODUCT_VERSION}");
            println!(
                "Development source foundation; no supported download workflow is available yet."
            );
        }
    }
}
