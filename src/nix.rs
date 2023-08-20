use std::collections::HashMap;
use std::io::Error;
use std::process::Command;

use serde::{Deserialize, Deserializer};

// This is a special file used By NixOS to represent the derivations
// that were used to build the current system.
const CURRENT_SYSTEM_PATH: &str = "/run/current-system";

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Clone)]
pub enum DerivationBuilder {
    FetchURL,
    Bash,
    Busybox,
    Unknown,
}

impl DerivationBuilder {
    pub fn from_string(builder: &str) -> Result<DerivationBuilder, String> {
        if builder == "builtin:fetchurl" {
            return Ok(DerivationBuilder::FetchURL);
        }
        if builder.ends_with("/bin/bash") {
            return Ok(DerivationBuilder::Bash);
        }
        if builder.ends_with("busybox") {
            return Ok(DerivationBuilder::Busybox);
        }
        Ok(DerivationBuilder::Unknown)
        // Here I'd like to return an error when I'm developing, so that I could be aware of other
        // builders found in the wild.
        // Err(format!("Invalid derivation builder {}.", builder))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DerivationBuilder, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf = String::deserialize(deserializer)?;

        match DerivationBuilder::from_string(&buf) {
            Ok(b) => Ok(b),
            Err(e) => Err(e).map_err(serde::de::Error::custom),
        }
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Clone)]
pub struct Derivation {
    outputs: HashMap<String, Output>,

    #[serde(rename = "inputSrcs")]
    inputs_sources: Vec<String>,

    #[serde(rename = "inputDrvs")]
    input_derivations: HashMap<String, Vec<String>>,

    system: String,

    #[serde(deserialize_with = "DerivationBuilder::deserialize")]
    builder: DerivationBuilder,

    args: Vec<String>,

    env: HashMap<String, String>,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

pub type Derivations = HashMap<String, Derivation>;
pub type Packages = HashMap<String, Package>;

impl Derivation {
    pub fn get_derivations_for_current_system() -> Result<Derivations, Error> {
        Derivation::get_derivations(CURRENT_SYSTEM_PATH)
    }

    pub fn get_derivations(file_path: &str) -> Result<Derivations, Error> {
        let output = Command::new("nix")
            .arg("show-derivation")
            .arg("-r")
            .arg(file_path)
            .output()?;

        let flat_derivations: Derivations = serde_json::from_slice(&output.stdout)?;

        Ok(flat_derivations)
    }

    pub fn build_and_get_derivations(
        file_path: &str,
        derivation_ref: &str,
    ) -> Result<Derivations, Error> {
        let derivation_path = format!("{}#{}", file_path, derivation_ref);
        let output = Command::new("nix")
            .arg("build")
            .arg("--show-out-paths")
            .arg(derivation_path)
            .output()?;

        let flat_derivations: Derivations = serde_json::from_slice(&output.stdout)?;

        Ok(flat_derivations)
    }

    pub fn get_name(&self) -> Option<&String> {
        self.env.get("name")
    }

    // Returns the store path of the stdenv used.
    pub fn get_stdenv_path(&self) -> Option<&String> {
        self.env.get("stdenv")
    }

    // Returns the store path of the stdenv used.
    pub fn get_source_path(&self) -> Option<&String> {
        self.env.get("src")
    }

    // Returns the store path of the stdenv used.
    pub fn get_url(&self) -> Option<&String> {
        // There's also a `urls` field that we could use here.
        self.env.get("url")
    }
}

#[derive(Debug)]
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
pub fn get_packages() -> Result<Packages, String> {
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

    let raw_packages: Packages =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

    let mut packages: Packages = Packages::default();
    // Re-index the packages using the internal package name.
    for package in raw_packages.values() {
        packages.insert(package.name.to_string(), package.clone());
    }

    Ok(packages)
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Meta {
    pub packages: HashMap<String, PackageMeta>,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
pub struct Package {
    // name of the derivation
    pub name: String,

    // package name
    pub pname: String,

    // package version
    pub version: String,

    // name of the system for which this package was built
    pub system: String,

    // name of the output
    #[serde(rename = "outputName")]
    pub output_name: String,

    pub meta: PackageMeta,
}
impl Package {
    pub fn get_purl(&self) -> String {
        format!("pkg:nix/{}@{}", self.name, self.version)
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
pub struct PackageMeta {
    pub available: Option<bool>,

    pub broken: Option<bool>,

    pub insecure: Option<bool>,

    pub description: Option<String>,

    pub unfree: Option<bool>,

    pub unsupported: Option<bool>,

    pub homepage: Option<Homepage>,

    pub maintainers: Option<Vec<PackageMaintainer>>,

    pub license: Option<License>,
}
impl PackageMeta {
    pub fn get_licenses(&self) -> Vec<LicenseDetails> {
        vec![]
    }
    pub fn get_homepages(&self) -> Vec<String> {
        match &self.homepage {
            Some(h) => match h {
                Homepage::One(homepage) => vec![homepage.clone()],
                Homepage::Many(homepages) => homepages.clone(),
            },
            None => vec![],
        }
    }
}

pub fn get_package_for_derivation(derivation_name: &str, packages: &Packages) -> Option<Package> {
    if let Some(package) = packages.get(derivation_name) {
        return Some(package.clone());
    }
    None
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Homepage {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
pub struct PackageMaintainer {
    pub email: String,
    pub name: String,

    #[serde(rename = "github")]
    pub github_username: Option<String>,

    #[serde(rename = "githubId")]
    pub github_id: Option<u64>,
    // TODO also support GPG keys
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum License {
    One(PackageLicense),
    Many(Vec<PackageLicense>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum PackageLicense {
    // This is used for unknown licenses, or to list only the SPDX ID.
    Name(String),
    Details(LicenseDetails),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
pub struct LicenseDetails {
    pub free: Option<bool>,
    pub redistributable: Option<bool>,
    pub deprecated: Option<bool>,

    #[serde(rename = "shortName")]
    pub short_name: Option<String>,

    #[serde(rename = "fullName")]
    pub full_name: Option<String>,

    // Some licenses might not have an SPDX ID, for example if they are not
    // free (the `Unfree` license).
    #[serde(rename = "spdxId")]
    pub spdx_id: Option<String>,
}
