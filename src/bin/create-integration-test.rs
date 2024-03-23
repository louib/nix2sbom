#[macro_use]
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

    let mut derivations: nix2sbom::nix::Derivations = nix2sbom::nix::Derivations::default();

    derivations = nix2sbom::nix::Derivation::get_derivations(&args.file_path)?;

    let packages = nix2sbom::nix::get_packages(None, args.no_meta)?;

    // let package_graph = nix2sbom::nix::get_package_graph(&derivations, &packages);
    let package_graph = nix2sbom::nix::get_package_graph_next(&derivations, &packages);

    let package_graph_stats = package_graph.get_stats();

    let mut required_packages = nix2sbom::nix::Packages::default();
    for (derivation_path, derivation) in derivations.iter() {
        let derivation_name = match derivation.get_name() {
            Some(n) => n,
            None => continue,
        };
        if packages.contains_key(&derivation_name) {
            required_packages.insert(
                derivation_name.to_string(),
                packages.get(&derivation_name).unwrap().clone(),
            );
        }
    }
    let packages = required_packages;

    let sbom_dump = match nix2sbom::sbom::Format::CycloneDX
        .dump(&nix2sbom::sbom::SerializationFormat::JSON, &package_graph)
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return Ok(std::process::ExitCode::FAILURE);
        }
    };

    // Saving the fixtures so we can replay the test later.
    let target_dir = format!("./tests/fixtures/{}", args.name);

    std::fs::create_dir(&target_dir);

    let packages_file_path = format!("{}/packages.json", target_dir);
    let package_graph_file_path = format!("{}/package-graph.json", target_dir);
    let package_graph_stats_file_path = format!("{}/package-graph-stats.json", target_dir);
    let derivations_file_path = format!("{}/derivations.json", target_dir);
    // FIXME the sbom file is not deterministic yet, so we can't us it for the integration tests.
    // let sbom_file_path = format!("{}/sbom.json", target_dir);

    let mut packages_file = File::create(packages_file_path)?;
    packages_file.write_all(serde_json::to_string_pretty(&packages).unwrap().as_bytes());

    let mut derivations_file = File::create(derivations_file_path)?;
    derivations_file.write_all(serde_json::to_string_pretty(&derivations).unwrap().as_bytes());

    let mut package_graph_file = File::create(package_graph_file_path)?;
    package_graph_file.write_all(serde_json::to_string_pretty(&package_graph).unwrap().as_bytes());

    let mut package_graph_stats_file = File::create(package_graph_stats_file_path)?;
    package_graph_stats_file.write_all(
        serde_json::to_string_pretty(&package_graph_stats)
            .unwrap()
            .as_bytes(),
    );

    // let mut sbom_file = File::create(sbom_file_path)?;
    // sbom_file.write_all(sbom_dump.as_bytes());

    Ok(std::process::ExitCode::SUCCESS)
}
