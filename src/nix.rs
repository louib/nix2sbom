use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
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

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
pub struct DisplayOptions {
    pub print_stdenv: bool,
    pub print_exclude_list: Vec<String>,
}

pub fn is_stdenv(name: &str) -> bool {
    let stdenv_names = vec![
        "stdenv-linux",
        // TODO probably other stdenv- derivatives to add
        // to this list
        "acl",
        "db",
        "attr",
        "patch",
        "bzip2",
        "patchelf",
        "pkg-config",
        "gnum4",
        // "isl", ????
        // "gmp-with-cxx", ????
        "automake",
        "autoconf",
        "libtool",
        "libffi",
        "zlib",
        "bison",
        "which",
        // "expat", ????
        "unzip",
        "findutils",
        "flex",
    ];
    for stdenv_name in stdenv_names {
        if name.starts_with(stdenv_name) {
            return true;
        }
    }
    false
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

    // Returns the store path of the source
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
            match urls.split(" ").nth(0) {
                Some(u) => return Some(u.to_string()),
                None => return None,
            }
        }
        None
    }

    // Returns the store path of the stdenv used.
    pub fn get_urls(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        if let Some(url) = self.env.get("url") {
            for url in url.split(" ").collect::<Vec<_>>() {
                response.push(url.to_string());
            }
        }
        if let Some(urls) = self.env.get("urls") {
            for url in urls.split(" ").collect::<Vec<_>>() {
                response.push(url.to_string());
            }
        }
        response
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

    pub fn pretty_print(&self, base_indent: usize, display_options: &DisplayOptions) -> Vec<PrettyPrintLine> {
        let mut response: Vec<PrettyPrintLine> = vec![];
        for url in self.get_urls() {
            response.push(PrettyPrintLine::new(url, base_indent + 1));
            return response;
        }
        if let Some(name) = self.get_name() {
            response.push(PrettyPrintLine::new(name, base_indent + 1));
            return response;
        }
        response.push(PrettyPrintLine::new("unknown derivation?", base_indent + 1));
        response
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
pub fn get_packages(metadata_path: Option<String>) -> Result<Packages, String> {
    let mut content: Vec<u8> = vec![];
    if let Some(path) = metadata_path {
        log::info!("Using the package metadata from {}", &path);
        content = fs::read(path).map_err(|e| e.to_string())?;
    } else {
        log::info!("Getting the metadata for packages in the Nix store");
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
        content = output.stdout;
    }

    let raw_packages: Packages = serde_json::from_slice(&content).map_err(|e| e.to_string())?;

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
#[derive(Default)]
pub struct PackageURL {
    pub scheme: String,
    pub host: String,
    pub path: Vec<String>,
    pub query_params: HashMap<String, String>,
}

impl PackageURL {
    pub fn to_string(&self) -> String {
        let mut full_path = self.path.join("/");

        // TODO add the query params
        format!("{}://{}/{}", self.scheme, self.host, full_path)
    }
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
        // FIXME this should not be using the nix scope, which does not actually exist.
        // See https://github.com/package-url/purl-spec/blob/master/PURL-TYPES.rst
        // for the accepted scopes.
        format!("pkg:nix/{}@{}", self.name, self.version)
    }

    pub fn pretty_print(&self, base_indent: usize, display_options: &DisplayOptions) -> Vec<PrettyPrintLine> {
        let mut response: Vec<PrettyPrintLine> = vec![];
        response.push(PrettyPrintLine::new(&self.pname, base_indent));
        response.push(PrettyPrintLine::new(
            format!("purl: {}", &self.get_purl()),
            base_indent + 1,
        ));
        if self.meta.broken.unwrap_or(false) {
            response.push(PrettyPrintLine::new("broken: true", base_indent + 1));
        }
        if self.meta.insecure.unwrap_or(false) {
            response.push(PrettyPrintLine::new("insecure: true", base_indent + 1));
        }
        if self.meta.unfree.unwrap_or(false) {
            response.push(PrettyPrintLine::new("unfree: true", base_indent + 1));
        }
        if self.meta.unsupported.unwrap_or(false) {
            response.push(PrettyPrintLine::new("unsupported: true", base_indent + 1));
        }
        response
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

    pub maintainers: Option<PackageMaintainers>,

    pub license: Option<License>,
}
impl PackageMeta {
    pub fn get_maintainers(&self) -> Vec<PackageMaintainer> {
        match &self.maintainers {
            Some(h) => match h {
                PackageMaintainers::List(maintainers) => maintainers.clone(),
                PackageMaintainers::EmbeddedList(lists) => {
                    let mut maintainers: Vec<PackageMaintainer> = vec![];
                    for list in lists {
                        maintainers.append(&mut list.clone());
                    }
                    return maintainers;
                }
            },
            None => vec![],
        }
    }
    pub fn get_licenses(&self) -> Vec<PackageLicense> {
        match &self.license {
            Some(h) => match h {
                License::One(license) => vec![license.clone()],
                License::Many(licenses) => licenses.clone(),
            },
            None => vec![],
        }
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
#[serde(untagged)]
pub enum PackageMaintainers {
    List(Vec<PackageMaintainer>),
    // FIXME this syntax is not officially supported, and the only known instance
    // was fixed here https://github.com/NixOS/nixpkgs/commit/f14b6f553a7721b963cf10048adf35d08d5d0253
    EmbeddedList(Vec<Vec<PackageMaintainer>>),
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
#[derive(Default)]
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

impl PackageNode {
    pub fn pretty_print(
        &self,
        graph: &PackageGraph,
        base_indent: usize,
        display_options: &DisplayOptions,
    ) -> Vec<PrettyPrintLine> {
        let mut lines: Vec<PrettyPrintLine> = vec![];

        for line in self.package.pretty_print(base_indent, display_options) {
            lines.push(line);
        }
        for line in self.main_derivation.pretty_print(base_indent, display_options) {
            lines.push(line);
        }
        if self.sources.len() != 0 {
            lines.push(PrettyPrintLine::new("sources:", base_indent + 1));
            for source in &self.sources {
                for line in source.pretty_print(base_indent + 1, display_options) {
                    lines.push(line);
                }
            }
        }
        if self.patches.len() != 0 {
            lines.push(PrettyPrintLine::new("patches:", base_indent + 1));
            for patch in &self.patches {
                for line in patch.pretty_print(base_indent + 1, display_options) {
                    lines.push(line);
                }
            }
        }
        if self.children.len() != 0 {
            for child_package_derivation_path in self.children.iter() {
                let child_package = match graph.get(child_package_derivation_path) {
                    Some(p) => p,
                    None => {
                        log::warn!(
                            "Could not get package in package graph for {}",
                            &child_package_derivation_path
                        );
                        continue;
                    }
                };
                if !display_options.print_stdenv && is_stdenv(child_package.main_derivation.get_name().unwrap())
                {
                    continue;
                }

                for line in child_package.pretty_print(&graph, base_indent + 1, display_options) {
                    lines.push(line);
                }
            }
        }
        lines
    }
}

pub type PackageGraph = HashMap<String, PackageNode>;

fn add_visited_children(
    package_node: &PackageNode,
    package_graph: &PackageGraph,
    visited_children: &mut HashSet<String>,
) {
    for child_derivation_path in &package_node.children {
        visited_children.insert(child_derivation_path.to_string());
        let child_package = match package_graph.get(child_derivation_path) {
            Some(p) => p,
            None => {
                log::warn!(
                    "Could not get package in package graph for {}",
                    &child_derivation_path
                );
                continue;
            }
        };

        add_visited_children(&child_package, &package_graph, visited_children);
    }
}

pub fn pretty_print_package_graph(
    package_graph: &PackageGraph,
    base_indent: usize,
    display_options: &DisplayOptions,
) -> String {
    let mut lines: Vec<PrettyPrintLine> = vec![];
    let mut response = "".to_string();

    let mut visited_children: HashSet<String> = HashSet::default();
    for (derivation_path, package_node) in package_graph {
        for child_derivation_path in &package_node.children {
            let child = package_graph.get(child_derivation_path).unwrap().clone();
            add_visited_children(child, &package_graph, &mut visited_children);
        }
    }

    for (derivation_path, package_node) in package_graph {
        if !display_options.print_stdenv && is_stdenv(package_node.main_derivation.get_name().unwrap()) {
            continue;
        }
        for line in package_node.pretty_print(package_graph, base_indent, display_options) {
            lines.push(line);
        }
    }

    for line in lines {
        response += &line.to_string();
        response += "\n";
    }
    response
}

// Small struct to make it easier to pretty-print the
// internal representation for the package graph.
#[derive(Debug)]
struct PrettyPrintLine {
    pub indent_level: usize,
    pub line: String,
}
impl PrettyPrintLine {
    pub fn new<S: AsRef<str>>(line: S, indent_level: usize) -> PrettyPrintLine {
        PrettyPrintLine {
            line: line.as_ref().to_string(),
            indent_level,
        }
    }

    pub fn to_string(&self) -> String {
        "  ".repeat(self.indent_level) + &self.line
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_package_metadata() {
        const package_metadata: &str = r###"
          {
            "name": "0ad-0.0.26",
            "pname": "0ad",
            "version": "0.0.26",
            "system": "x86_64-linux",
            "outputName": "out",
            "outputs": {
              "out": null
            },
            "meta": {
              "available": true,
              "broken": false,
              "description": "A free, open-source game of ancient warfare",
              "homepage": "https://play0ad.com/",
              "insecure": false,
              "license": [
                {
                  "deprecated": true,
                  "free": true,
                  "fullName": "GNU General Public License v2.0",
                  "redistributable": true,
                  "shortName": "gpl2",
                  "spdxId": "GPL-2.0",
                  "url": "https://spdx.org/licenses/GPL-2.0.html"
                },
                {
                  "deprecated": true,
                  "free": true,
                  "fullName": "GNU Lesser General Public License v2.1",
                  "redistributable": true,
                  "shortName": "lgpl21",
                  "spdxId": "LGPL-2.1",
                  "url": "https://spdx.org/licenses/LGPL-2.1.html"
                },
                {
                  "deprecated": false,
                  "free": true,
                  "fullName": "MIT License",
                  "redistributable": true,
                  "shortName": "mit",
                  "spdxId": "MIT",
                  "url": "https://spdx.org/licenses/MIT.html"
                },
                {
                  "deprecated": false,
                  "free": true,
                  "fullName": "Creative Commons Attribution Share Alike 3.0",
                  "redistributable": true,
                  "shortName": "cc-by-sa-30",
                  "spdxId": "CC-BY-SA-3.0",
                  "url": "https://spdx.org/licenses/CC-BY-SA-3.0.html"
                },
                {
                  "deprecated": false,
                  "free": true,
                  "fullName": "zlib License",
                  "redistributable": true,
                  "shortName": "zlib",
                  "spdxId": "Zlib",
                  "url": "https://spdx.org/licenses/Zlib.html"
                }
              ],
              "maintainers": [
                {
                  "email": "nixpkgs@cvpetegem.be",
                  "github": "chvp",
                  "githubId": 42220376,
                  "matrix": "@charlotte:vanpetegem.me",
                  "name": "Charlotte Van Petegem"
                }
              ],
              "name": "0ad-0.0.26",
              "outputsToInstall": [
                "out"
              ],
              "platforms": [
                "aarch64-linux",
                "armv5tel-linux",
                "armv6l-linux",
                "armv7a-linux",
                "armv7l-linux",
                "m68k-linux",
                "microblaze-linux",
                "microblazeel-linux",
                "mipsel-linux",
                "mips64el-linux",
                "powerpc64-linux",
                "powerpc64le-linux",
                "riscv32-linux",
                "riscv64-linux",
                "s390-linux",
                "s390x-linux",
                "x86_64-linux"
              ],
              "unfree": false,
              "unsupported": false
            }
          }
        "###;
        let package: Package = serde_json::from_str(package_metadata).unwrap();
        assert_eq!(package.name, "0ad-0.0.26");
    }

    #[test]
    pub fn parse_package_metadata_embedded_maintainers_list() {
        // This parsing issue was raised in https://github.com/louib/nix2sbom/issues/10
        const package_metadata: &str = r###"
          {
            "meta": {
              "available": true,
              "broken": false,
              "description": "A parser generator for building parsers from grammars",
              "homepage": "https://javacc.github.io/javacc",
              "insecure": false,
              "license": {
                "deprecated": false,
                "free": true,
                "fullName": "BSD 2-clause \"Simplified\" License",
                "redistributable": true,
                "shortName": "bsd2",
                "spdxId": "BSD-2-Clause",
                "url": "https://spdx.org/licenses/BSD-2-Clause.html"
              },
              "maintainers": [
                [
                  {
                    "email": "limeytexan@gmail.com",
                    "github": "limeytexan",
                    "githubId": 36448130,
                    "name": "Michael Brantley"
                  }
                ]
              ],
              "name": "javacc-7.0.10",
              "outputsToInstall": [
                "out"
              ],
              "unfree": false,
              "unsupported": false
            },
            "name": "javacc-7.0.10",
            "outputName": "out",
            "outputs": {
              "out": null
            },
            "pname": "javacc",
            "system": "x86_64-linux",
            "version": "7.0.10"
          }
        "###;
        let package: Package = serde_json::from_str(package_metadata).unwrap();
        assert_eq!(package.name, "javacc-7.0.10");
        assert_eq!(package.meta.get_maintainers().len(), 1);
    }
}
