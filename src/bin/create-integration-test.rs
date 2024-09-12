extern crate clap;

use std::fs::File;
use std::io::Write;

use clap::Parser;

#[derive(Parser)]
struct CreateIntegrationTest {
    /// Name of the new integration test
    name: String,

    /// The path of the nix file to create the integration test from
    file_path: String,

    /// Do not use the metadata from the store to generate the SBOM.
    #[clap(long, short)]
    no_meta: bool,
}

fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    let args = CreateIntegrationTest::parse();

    let derivations = nix2sbom::nix::Derivation::get_derivations(&args.file_path)?;

    let packages = nix2sbom::nix::Packages::default();
    let mut package_graph = nix2sbom::nix::get_package_graph_next(&derivations, &packages);
    let package_groups = package_graph.get_package_groups();

    // Saving the fixtures so we can replay the test later.
    let target_dir = format!("./tests/fixtures/{}", args.name);

    std::fs::create_dir(&target_dir)?;

    let derivations_file_path = format!("{}/derivations.json", target_dir);
    let package_groups_file_path = format!("{}/package-groups.yaml", target_dir);

    let mut derivations_file = File::create(derivations_file_path)?;
    derivations_file.write_all(serde_json::to_string_pretty(&derivations).unwrap().as_bytes())?;

    let mut package_groups_file = File::create(package_groups_file_path)?;
    package_groups_file.write_all(serde_yaml::to_string(&package_groups).unwrap().as_bytes())?;

    Ok(std::process::ExitCode::SUCCESS)
}
