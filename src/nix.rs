use std::collections::HashMap;
use std::io::Error;
use std::process::Command;

use serde::{Deserialize, Serialize};

// This is a special file used By NixOS to represent the derivations
// that were used to build the current system.
const CURRENT_SYSTEM_PATH: &str = "/run/current-system";

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
    pub fn get_derivations_for_current_system() -> Result<Vec<Derivation>, Error> {
        Derivation::get_derivations(CURRENT_SYSTEM_PATH)
    }

    pub fn get_derivations(file_path: &str) -> Result<Vec<Derivation>, Error> {
        let output = Command::new("nix")
            .arg("show-derivation")
            .arg("-r")
            .arg(file_path)
            .output()?;

        let flat_derivations: Vec<Derivation> = serde_json::from_slice(&output.stdout)?;

        Ok(flat_derivations)
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
        .arg(".*")
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
