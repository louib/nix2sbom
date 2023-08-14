extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use std::io::Error;
use std::process::Command;

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
struct Derivation {
    outputs: HashMap<String, Output>,

    #[serde(rename = "inputSrcs")]
    inputs_sources: Vec<String>,

    #[serde(rename = "inputDrvs")]
    input_derivations: HashMap<String, Vec<String>>,

    system: String,

    builder: String,

    args: Vec<String>,

    env: HashMap<String, String>,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl Derivation {
    pub fn parse_from_file(file: &str) -> Result<Vec<u8>, Error> {
        let output = Command::new("nix")
            .arg("show-derivation")
            .arg("-r")
            .arg(file)
            .output()?;

        Ok(output.stdout)
    }
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
struct Output {
    path: String,
}

pub fn get_dependencies(path: &str) -> Vec<String> {
    // TODO nix-store -qR /an/executable/path
    vec![]
}

// Get the derivation path associated with a store object
pub fn get_derivation_path(store_path: &str) -> String {
    // TODO nix-store -qd store_path
    "".to_string()
}
pub fn get_packages() -> Result<HashMap<String, PackageMeta>, String> {
    // There is currently no way with Nix to generate the meta information
    // only for a single derivation. We need to generate the meta for
    // all the derivations in the store and then extract the information
    // we want from the global meta database.
    let output = Command::new("nix-env")
        .arg("-q")
        .arg("-a")
        .arg("--meta")
        .arg("--json")
        .arg("'.*'")
        .output()
        .map_err(|e| e.to_string())?;

    let packages: HashMap<String, PackageMeta> =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    Ok(packages)
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Meta {
    pub packages: HashMap<String, PackageMeta>,
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Package {
    // name of the derivation
    name: String,

    // package name
    pname: String,

    // package version
    version: String,

    // name of the system for which this package was built
    system: String,

    // name of the output
    #[serde(rename = "outputName")]
    output_name: String,
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct PackageMeta {
    available: Option<bool>,

    broken: Option<bool>,

    insecure: Option<bool>,

    description: Option<String>,

    unfree: Option<bool>,

    unsupported: Option<bool>,

    homepage: Option<String>,
}
