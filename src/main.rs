// TODO tune built-in attributes
// From https://doc.rust-lang.org/reference/items/modules.html#attributes-on-modules
// The built-in attributes that have meaning on a module are cfg, deprecated, doc,
// the lint check attributes, path, and no_implicit_prelude.
// Modules also accept macro attributes.

#[macro_use]
extern crate clap;

use clap::Parser;

mod cyclone_dx;
mod logger;
mod nix;
mod sbom;

/// nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation
#[derive(Parser)]
#[clap(name = "nix2sbom")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation", long_about = None)]
struct NixToSBOM {}

fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    logger::init();
    log::info!("Getting the meta info for packages in the Nix store");
    let packages = crate::nix::get_packages()?;
    log::debug!("Found {} packages in the Nix store", packages.len());

    Ok(std::process::ExitCode::SUCCESS)
}
