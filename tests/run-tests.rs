use std::collections::BTreeMap;

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
    let package_nodes_file_path = format!("{}/package-nodes.json", path.display());

    let file = File::open(derivations_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let derivations: nix2sbom::nix::Derivations = serde_json::from_str(&contents).unwrap();

    let file = File::open(package_nodes_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let package_graph: BTreeMap<String, nix2sbom::nix::PackageNode> = serde_json::from_str(&contents).unwrap();

    let packages = nix2sbom::nix::Packages::default();
    let mut expected_package_graph = nix2sbom::nix::get_package_graph(&derivations);
    expected_package_graph.transform(&packages).unwrap();

    assert_eq!(expected_package_graph.nodes_next, package_graph);
}
