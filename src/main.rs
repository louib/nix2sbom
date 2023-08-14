// TODO tune built-in attributes
// From https://doc.rust-lang.org/reference/items/modules.html#attributes-on-modules
// The built-in attributes that have meaning on a module are cfg, deprecated, doc,
// the lint check attributes, path, and no_implicit_prelude.
// Modules also accept macro attributes.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

use clap::Parser;

mod logger;
mod nix;
mod sbom;

/// nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation
#[derive(Parser)]
#[clap(name = "nix2sbom")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation", long_about = None)]
struct NixToSBOM {}

fn main() {
    logger::init();
    println!("Hello, world!");
}
