use std::collections::{BTreeSet, HashMap, HashSet};
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
    pub outputs: HashMap<String, Output>,

    #[serde(rename = "inputSrcs")]
    pub inputs_sources: Vec<String>,

    #[serde(rename = "inputDrvs")]
    pub input_derivations: HashMap<String, Vec<String>>,

    pub system: String,

    #[serde(deserialize_with = "DerivationBuilder::deserialize")]
    pub builder: DerivationBuilder,

    pub args: Vec<String>,

    pub env: HashMap<String, String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
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

    pub fn build_and_get_derivations(file_path: &str, derivation_ref: &str) -> Result<Derivations, Error> {
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
    pub fn get_url(&self) -> Option<String> {
        if let Some(url) = self.env.get("url") {
            return Some(url.to_owned());
        }
        if let Some(urls) = self.env.get("urls") {
            // FIXME I'm not sure that this is the right separator!!
            // FIXME How whould we handle multiple URLs???
            match urls.split(",").nth(0) {
                Some(u) => return Some(u.to_string()),
                None => return None,
            }
        }
        None
    }

    // Returns the out path of the patches for that derivation
    pub fn get_patches(&self) -> Vec<String> {
        if let Some(patches) = self.env.get("patches") {
            let mut response: Vec<String> = vec![];
            for patch in patches.split(" ") {
                response.push(patch.to_string());
            }
            return response;
        }
        vec![]
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Clone)]
pub struct Output {
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

    let raw_packages: Packages = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

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

#[derive(Debug)]
pub struct PackageNode {
    pub main_derivation: Derivation,

    pub package: Package,

    pub sources: Vec<Derivation>,

    pub patches: Vec<Derivation>,

    pub children: HashSet<String>,
}

pub type PackageGraph = HashMap<String, PackageNode>;

pub fn get_package_graph(
    derivations: &crate::nix::Derivations,
    packages: &crate::nix::Packages,
) -> PackageGraph {
    let mut response = PackageGraph::default();

    for (derivation_path, derivation) in derivations.iter() {
        let derivation_name = match derivation.get_name() {
            Some(n) => n,
            None => {
                log::warn!("Found derivation without a name at {}", derivation_path);
                continue;
            }
        };
        let package = match packages.get(derivation_name) {
            Some(p) => p,
            None => continue,
        };
        println!("{} is a main package", derivation_path);
        let mut current_node = PackageNode {
            package: package.clone(),
            main_derivation: derivation.clone(),
            children: HashSet::default(),
            sources: vec![],
            patches: vec![],
        };
        let current_node_patches = derivation.get_patches();

        let mut child_derivation_paths: BTreeSet<String> = BTreeSet::default();
        for input_derivation_path in derivation.input_derivations.keys() {
            child_derivation_paths.insert(input_derivation_path.clone());
        }

        let mut visited_derivations: HashSet<String> = HashSet::default();

        while child_derivation_paths.len() != 0 {
            let child_derivation_path = child_derivation_paths.pop_last().unwrap();
            if visited_derivations.contains(&child_derivation_path) {
                continue;
            }
            visited_derivations.insert(child_derivation_path.clone());

            let child_derivation = derivations.get(&child_derivation_path).unwrap();
            let child_derivation_name = match child_derivation.get_name() {
                Some(n) => n,
                None => {
                    log::trace!("Derivation without a name {:?}", &child_derivation);
                    // FIXME this is ugly. We should just add the input derivations in the graph
                    // traversal list and move on instead of using a placeholder value.
                    "NOT_AN_ACTUAL_NAME"
                }
            };
            if child_derivation_name != "source" && packages.get(child_derivation_name).is_some() {
                log::info!("Found a child derivation that is a main package!!!!!!");
                current_node.children.insert(child_derivation_path.to_string());
                // FIXME should we really continue here? Are there derivations that define both a
                // package meta and urls to fetch?
                continue;
            } else if child_derivation.env.get("src").is_some() {
                // The `src` attribute is defined by the mkDerivation function, so in theory we
                // should always find the package in the meta dictionary if the src attribute
                // is defined.
                // FIXME We should still consider those as Packages even if we don't have the meta
                // information on them
                continue;
            }
            if child_derivation.get_url().is_some() {
                if child_derivation.env.get("out").is_some()
                    && current_node_patches.contains(child_derivation.env.get("out").unwrap())
                {
                    current_node.patches.push(child_derivation.clone());
                } else {
                    current_node.sources.push(child_derivation.clone());
                }
            }

            for input_derivation_path in child_derivation.input_derivations.keys() {
                child_derivation_paths.insert(input_derivation_path.clone());
            }
        }
        response.insert(derivation_path.clone(), current_node);
    }
    response
}
