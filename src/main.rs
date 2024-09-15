// TODO tune built-in attributes
// From https://doc.rust-lang.org/reference/items/modules.html#attributes-on-modules
// The built-in attributes that have meaning on a module are cfg, deprecated, doc,
// the lint check attributes, path, and no_implicit_prelude.
// Modules also accept macro attributes.

extern crate clap;

use clap::Parser;

/// nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation
#[derive(Parser)]
#[clap(name = nix2sbom::consts::PROJECT_NAME)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "nix2sbom extracts the SBOM (Software Bill of Materials) from a Nix derivation", long_about = None)]
struct NixToSBOM {
    /// Reference to a nix derivation. The reference includes the path to the nix
    /// file and the path of the nix derivation within the file.
    /// Example: /path/to/default.nix#derivation
    nix_ref: Option<String>,

    /// Output format for the SBOM manifest. Defaults to cdx (CycloneDX).
    #[clap(short, long)]
    format: Option<String>,

    /// Which format to use for serializing the SBOM. CycloneDX supports yaml and json.
    #[clap(short, long)]
    serialization_format: Option<String>,

    /// Path of an existing package metadata file.
    ///
    /// This file can be generated by using the following command:
    /// nix-env -q -a --meta --json '.*'
    #[clap(long)]
    metadata_path: Option<String>,

    /// Use the metadata from the store to help generating the SBOM.
    #[clap(long, short)]
    meta: bool,

    /// Do not pretty print the generated SBOM manifest
    #[clap(long)]
    no_pretty: bool,

    /// Include only the runtime dependencies in the SBOM.
    #[clap(long, short)]
    runtime_only: bool,

    /// Generate a SBOM for the current system.
    #[clap(long, short)]
    current_system: bool,
}

fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    nix2sbom::logger::init();
    let args = NixToSBOM::parse();

    let output_format = match args.format {
        Some(f) => match nix2sbom::sbom::Format::from_string(&f) {
            Some(f) => f,
            None => {
                eprintln!("Invalid format {}", &f);
                return Ok(std::process::ExitCode::FAILURE);
            }
        },
        None => nix2sbom::sbom::Format::default(),
    };

    let serialization_format = match args.serialization_format {
        Some(f) => match nix2sbom::sbom::SerializationFormat::from_string(&f) {
            Some(f) => f,
            None => {
                eprintln!("Invalid serialization format {}", &f);
                return Ok(std::process::ExitCode::FAILURE);
            }
        },
        None => output_format.get_default_serialization_format(),
    };

    let derivations: nix2sbom::nix::Derivations = if let Some(nix_ref) = args.nix_ref {
        log::info!("Getting the derivations from {}", &nix_ref);
        nix2sbom::nix::Derivation::get_derivations(&nix_ref)?
    } else if args.current_system {
        log::info!("Getting the derivations from the current system");
        nix2sbom::nix::Derivation::get_derivations_for_current_system()?
    } else {
        eprintln!("Error: Must provide a file or use the --curent-system argument");
        return Ok(std::process::ExitCode::FAILURE);
    };
    log::info!("Found {} derivations", derivations.len());

    let packages = nix2sbom::nix::get_packages(args.metadata_path, !args.meta)?;
    log::debug!("Found {} packages in the Nix store", packages.len());

    log::info!("Building the package graph");
    let mut package_graph = nix2sbom::nix::get_package_graph(&derivations);
    log::info!("{} nodes in the package graph", package_graph.nodes.len());
    log::debug!(
        "{} root nodes in the package graph",
        package_graph.root_nodes.len()
    );
    package_graph.transform(&packages)?;

    log::debug!("Creating the SBOM");

    let mut dump_options = nix2sbom::nix::DumpOptions::default();
    dump_options.runtime_only = args.runtime_only;
    if args.no_pretty {
        dump_options.pretty = Some(false);
    };

    let sbom_dump = match output_format.dump(&serialization_format, &package_graph, &dump_options) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return Ok(std::process::ExitCode::FAILURE);
        }
    };

    println!("{}", sbom_dump);

    Ok(std::process::ExitCode::SUCCESS)
}
