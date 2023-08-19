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
struct NixToSBOM {
    /// Path of the file to extract a SBOM manifest from.
    #[clap(long, short)]
    file_path: Option<String>,

    /// Output format for the SBOM manifest. Defaults to CycloneDX
    #[clap(long)]
    format: Option<String>,

    /// Generate a SBOM for the current system.
    #[clap(long, short)]
    current_system: bool,
}

fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    logger::init();
    let args = NixToSBOM::parse();

    let mut derivations: crate::nix::Derivations = crate::nix::Derivations::default();
    if let Some(file_path) = args.file_path {
        log::info!("Getting the derivations from file {}.", &file_path);
        derivations = nix::Derivation::get_derivations(&file_path)?;
    } else if args.current_system {
        log::info!("Getting the derivations from the current system.");
        derivations = nix::Derivation::get_derivations_for_current_system()?;
    } else {
        eprintln!("Error: Must provide a file or use the --curent-system argument.");
        return Ok(std::process::ExitCode::FAILURE);
    }
    log::info!("Found {} derivations", derivations.len());

    log::info!("Getting the metadata for packages in the Nix store");
    let packages = crate::nix::get_packages()?;
    log::debug!("Found {} packages in the Nix store", packages.len());

    let output_format = match args.format {
        Some(f) => match crate::sbom::Format::from_string(&f) {
            Some(f) => f,
            None => {
                eprintln!("Invalid format {}", &f);
                return Ok(std::process::ExitCode::FAILURE);
            }
        },
        None => crate::sbom::Format::default(),
    };

    match output_format {
        crate::sbom::Format::CycloneDX => {
            let output = crate::cyclone_dx::dump(&derivations, &packages);
        }
        crate::sbom::Format::SPDX => {
            eprintln!(
                "{} is not supported yet.",
                crate::sbom::Format::SPDX.to_pretty_name()
            );
            return Ok(std::process::ExitCode::FAILURE);
        }
    };

    Ok(std::process::ExitCode::SUCCESS)
}
