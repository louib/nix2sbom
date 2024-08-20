use rstest::*;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use nix2sbom;

#[rstest]
fn for_each_file(#[files("tests/fixtures/*")] path: PathBuf) {
    if path.display().to_string().contains("DO_NOT_DELETE.txt") {
        return;
    }

    let derivations_file_path = format!("{}/derivations.json", path.display());
    let package_graph_file_path = format!("{}/package-graph.json", path.display());

    let file = File::open(derivations_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let derivations: nix2sbom::nix::Derivations = serde_json::from_str(&contents).unwrap();

    let file = File::open(package_graph_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let package_graph: nix2sbom::nix::PackageGraph = serde_json::from_str(&contents).unwrap();

    let packages = nix2sbom::nix::Packages::default();
    let expected_package_graph = nix2sbom::nix::get_package_graph_next(&derivations, &packages);
    println!("{}", derivations.len());

    assert_eq!(expected_package_graph, package_graph);
    assert_eq!(
        expected_package_graph.get_patches_count(),
        package_graph.get_patches_count()
    );
}
