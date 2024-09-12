use rstest::*;
use std::collections::BTreeMap;
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
    let package_groups_file_path = format!("{}/package-groups.yaml", path.display());

    let file = File::open(derivations_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let derivations: nix2sbom::nix::Derivations = serde_json::from_str(&contents).unwrap();

    let file = File::open(package_groups_file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let package_groups: BTreeMap<String, nix2sbom::nix::PackageGroup> =
        serde_yaml::from_str(&contents).unwrap();

    let packages = nix2sbom::nix::Packages::default();
    let mut expected_package_graph = nix2sbom::nix::get_package_graph_next(&derivations, &packages);
    let expected_package_groups = expected_package_graph.get_package_groups();

    assert_eq!(expected_package_groups.len(), package_groups.len());
    for (package_group_name, package_group) in expected_package_groups.iter() {
        let expected_package_group = package_groups.get(package_group_name).unwrap();
        assert_eq!(expected_package_group, package_group);
    }
}
