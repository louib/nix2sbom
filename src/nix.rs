use std::error::Error;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::process::Command;

use serde::{Deserialize, Deserializer, Serialize};

// This is a special file used By NixOS to represent the derivations
// that were used to build the current system.
const CURRENT_SYSTEM_PATH: &str = "/run/current-system";

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[derive(PartialEq)]
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
    pub print_only_purl: bool,
    pub max_depth: Option<usize>,
}

pub enum PackageScope {
    PERL,
    PYTHON,
    RUBY,
}

pub fn is_stdenv(name: &str) -> bool {
    let stdenv_names = vec![
        "stdenv-linux",
        // TODO probably other stdenv- derivatives to add
        // to this list
        "acl",
        "autoconf",
        "automake",
        "attr",
        "binutils",
        "bison",
        "bzip2",
        "db",
        // "expat", ????
        "findutils",
        "flex",
        "gnum4",
        "gettext",
        // gcc???
        // "gmp-with-cxx", ????
        // "isl", ????
        "perl",
        "patch",
        "patchelf",
        "pkg-config",
        "texinfo",
        "libtool",
        "libffi",
        "unzip",
        "zlib",
        "which",
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
        if builder.ends_with("/bin/bash") || builder == "Bash" {
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
#[derive(Serialize)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct InputDerivationDetails {
    outputs: Vec<String>,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum InputDerivation {
    List(Vec<String>),
    Details(InputDerivationDetails),
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct Derivation {
    pub outputs: HashMap<String, Output>,

    #[serde(rename = "inputSrcs")]
    pub inputs_sources: Vec<String>,

    #[serde(rename = "inputDrvs")]
    pub input_derivations: HashMap<String, InputDerivation>,

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
    pub fn get_derivations_for_current_system() -> Result<Derivations, Box<dyn Error>> {
        Derivation::get_derivations(CURRENT_SYSTEM_PATH)
    }

    pub fn get_scope(&self) -> Option<PackageScope> {
        if self.env.get("fullperl").is_some() {
            return Some(PackageScope::PERL);
        }
        None
    }

    pub fn get_derivations(file_path: &str) -> Result<Derivations, Box<dyn Error>> {
        let output = Command::new("nix")
            .arg("derivation")
            .arg("show")
            // FIXME we might want to disable impure by default.
            .arg("--impure")
            .arg("-r")
            .arg(file_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();
            return Err(format!("Could not get derivations from {}: {}", &file_path, &stderr).into());
        }

        let flat_derivations: Derivations = serde_json::from_slice(&output.stdout)?;

        Ok(flat_derivations)
    }

    pub fn to_json(&self) -> Result<String, String> {
        return serde_json::to_string_pretty(self).map_err(|e| e.to_string());
    }

    pub fn build_and_get_derivations(
        file_path: &str,
        derivation_ref: &str,
    ) -> Result<Derivations, Box<dyn Error>> {
        let derivation_path = format!("{}#{}", file_path, derivation_ref);
        let output = Command::new("nix")
            .arg("build")
            // FIXME we might want to disable impure by default.
            .arg("--impure")
            .arg("--show-out-paths")
            .arg(derivation_path)
            .output()?;

        let flat_derivations: Derivations = serde_json::from_slice(&output.stdout)?;

        Ok(flat_derivations)
    }

    pub fn get_name(&self) -> Option<String> {
        if let Some(pname) = self.env.get("pname") {
            return Some(pname.to_string());
        }
        if let Some(name) = self.env.get("name") {
            if let Some(version) = self.get_version_from_env() {
                if name.contains(&version) {
                    let package_version_suffix = "-".to_string() + &version;
                    return Some(name.replace(&package_version_suffix, ""));
                }
            }
            if name != "source" {
                return Some(name.to_string());
            }
        }

        for url in self.get_urls() {
            if let Some(project_name) = crate::utils::get_project_name_from_generic_url(&url) {
                return Some(project_name.to_string());
            }
            if let Some(project_name) = crate::utils::get_project_name_from_archive_url(&url) {
                return Some(project_name.to_string());
            }
        }

        None
    }

    // Returns the store path of the stdenv used.
    pub fn get_stdenv_path(&self) -> Option<&String> {
        self.env.get("stdenv")
    }

    // Returns the store path of the source
    pub fn get_source_path(&self) -> Option<&String> {
        self.env.get("src")
    }

    // Returns the main url of the derivation
    pub fn get_url(&self) -> Option<String> {
        let urls = self.get_urls();
        return urls.get(0).cloned();
    }

    // Returns the store path of the stdenv used.
    pub fn get_urls(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        if let Some(url) = self.env.get("url") {
            for url in url.split(" ").collect::<Vec<_>>() {
                response.push(crate::mirrors::translate_url(url));
            }
        }
        if let Some(urls) = self.env.get("urls") {
            for url in urls.split(" ").collect::<Vec<_>>() {
                response.push(crate::mirrors::translate_url(url));
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

    pub fn pretty_print(&self, depth: usize, display_options: &DisplayOptions) -> Vec<PrettyPrintLine> {
        let mut response: Vec<PrettyPrintLine> = vec![];
        for url in self.get_urls() {
            response.push(PrettyPrintLine::new(url, depth + 1));
            return response;
        }
        if let Some(name) = self.get_name() {
            response.push(PrettyPrintLine::new(name, depth + 1));
            return response;
        }
        response.push(PrettyPrintLine::new("unknown derivation?", depth + 1));
        response
    }

    // Get the version but only if found directly from the env dictionary of
    // the derivation. This is used because the other techniques used to detect the
    // version (like parsing the URLs) are less reliable. We need a high certainty that
    // this is the correct version if we want to use the version to extract the package name
    // (pname) from the name of the derivation.
    fn get_version_from_env(&self) -> Option<String> {
        if let Some(revision) = self.env.get("rev") {
            if revision.starts_with("v") {
                return Some(revision[1..].to_string());
            }
            return Some(revision.to_string());
        }
        if let Some(version) = self.env.get("version") {
            return Some(version.to_string());
        }
        None
    }

    pub fn get_version(&self) -> Option<String> {
        if let Some(version) = self.get_version_from_env() {
            return Some(version);
        }
        for url in self.get_urls() {
            if let Some(commit_sha) = crate::utils::get_git_sha_from_archive_url(&url) {
                return Some(commit_sha);
            }
            if let Some(version) = crate::utils::get_semver_from_archive_url(&url) {
                return Some(version);
            }
        }
        let pname = match self.env.get("pname") {
            Some(n) => n,
            None => return None,
        };
        let name = match self.env.get("name") {
            Some(n) => n,
            None => return None,
        };
        if name.contains(pname) {
            let package_name_prefix = pname.to_string() + "-";
            return Some(name.replace(&package_name_prefix, ""));
        }
        None
    }

    pub fn is_inline_script(&self) -> bool {
        self.env.get("text").is_some()
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[derive(PartialEq)]
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
pub fn get_packages(metadata_path: Option<String>, no_meta: bool) -> Result<Packages, String> {
    let mut packages: Packages = Packages::default();

    if no_meta {
        return Ok(packages);
    }

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

    // Re-index the packages using the internal package name.
    for package in raw_packages.values() {
        packages.insert(package.name.to_string(), package.clone());
    }

    Ok(packages)
}

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
pub struct Meta {
    pub packages: HashMap<String, PackageMeta>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct PackageURL {
    pub scheme: String,
    pub host: String,
    pub version: Option<String>,
    pub path: Vec<String>,
    pub query_params: HashMap<String, String>,
}

impl PackageURL {
    pub fn to_string(&self) -> String {
        let mut response = format!("{}://", self.scheme);
        response += &self.host.clone();

        let mut full_path = self.path.join("/");
        if !full_path.is_empty() {
            response += &full_path;
        }

        if let Some(version) = &self.version {
            response += &("@".to_string() + version);
        }
        response
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
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
    pub fn pretty_print(&self, depth: usize, display_options: &DisplayOptions) -> Vec<PrettyPrintLine> {
        let mut response: Vec<PrettyPrintLine> = vec![];
        if self.meta.broken.unwrap_or(false) {
            response.push(PrettyPrintLine::new("broken: true", depth + 1));
        }
        if self.meta.insecure.unwrap_or(false) {
            response.push(PrettyPrintLine::new("insecure: true", depth + 1));
        }
        if self.meta.unfree.unwrap_or(false) {
            response.push(PrettyPrintLine::new("unfree: true", depth + 1));
        }
        if self.meta.unsupported.unwrap_or(false) {
            response.push(PrettyPrintLine::new("unsupported: true", depth + 1));
        }
        response
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
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
#[derive(Serialize)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum Homepage {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum PackageMaintainers {
    List(Vec<PackageMaintainer>),
    // FIXME this syntax is not officially supported, and the only known instance
    // was fixed here https://github.com/NixOS/nixpkgs/commit/f14b6f553a7721b963cf10048adf35d08d5d0253
    EmbeddedList(Vec<Vec<PackageMaintainer>>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
pub struct PackageMaintainer {
    pub email: Option<String>,
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
#[derive(Serialize)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum License {
    One(PackageLicense),
    Many(Vec<PackageLicense>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum PackageLicense {
    // This is used for unknown licenses, or to list only the SPDX ID.
    Name(String),
    Details(LicenseDetails),
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
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
#[derive(PartialEq)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct PackageNode {
    pub main_derivation: Derivation,

    pub package: Option<Package>,

    pub sources: Vec<Derivation>,

    pub patches: BTreeSet<String>,

    pub children: BTreeSet<String>,
}

impl PackageNode {
    pub fn get_reachable_nodes_count(
        &self,
        package_nodes: &BTreeMap<String, PackageNode>,
        visited_children: &mut HashSet<String>,
    ) -> usize {
        let mut count = 1;
        for child_derivation_path in &self.children {
            if visited_children.contains(child_derivation_path) {
                continue;
            }
            let child_package = match package_nodes.get(child_derivation_path) {
                Some(p) => p,
                None => {
                    log::warn!(
                        "Could not get package in package graph for {}",
                        &child_derivation_path
                    );
                    continue;
                }
            };
            count += child_package.get_reachable_nodes_count(package_nodes, visited_children);
            visited_children.insert(child_derivation_path.to_string());
        }
        count
    }

    pub fn get_longest_path(
        &self,
        name: &str,
        package_nodes: &BTreeMap<String, PackageNode>,
        visited_children: &mut HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut longest_path = vec![];
        for child_derivation_path in &self.children {
            let path = match visited_children.get(child_derivation_path) {
                Some(p) => p.to_vec(),
                None => {
                    let child_package = package_nodes.get(child_derivation_path).unwrap();
                    child_package.get_longest_path(&child_derivation_path, package_nodes, visited_children)
                }
            };
            if path.len() > longest_path.len() {
                longest_path = path.to_vec();
            }
            visited_children.insert(child_derivation_path.to_string(), path.to_vec());
        }

        let mut response = vec![name.to_string()];
        response.append(&mut longest_path);
        response
    }

    pub fn get_name(&self) -> Option<String> {
        if let Some(p) = &self.package {
            if p.pname != "source" {
                return Some(p.pname.to_string());
            }
            if p.name != "source" {
                return Some(p.name.to_string());
            }
        }

        if let Some(name) = self.main_derivation.get_name() {
            return Some(name);
        }

        // FIXME I'm not sure we should rely on the sources to get the name here.
        for source in &self.sources {
            if let Some(source_name) = source.get_name() {
                if source_name != "source" {
                    return Some(source_name.to_string());
                }
            }
        }

        return None;
    }

    pub fn is_inline_script(&self) -> bool {
        self.main_derivation.is_inline_script()
    }

    pub fn get_version(&self) -> Option<String> {
        if let Some(p) = &self.package {
            if !p.version.is_empty() {
                return Some(p.version.to_string());
            }
        }

        return None;
    }

    pub fn get_purl(&self) -> PackageURL {
        let mut package_url = PackageURL::default();

        let mut name: Option<String> = self.get_name();
        if let Some(n) = &name {
            log::debug!("Found package name from source: {}", &n);
        } else {
            log::debug!(
                "Could not find package name anywhere for {}",
                &self.to_json().unwrap()
            );
            name = Some("unknown".to_string());
        }

        if name == Some("source".to_string()) {
            log::trace!("{}", self.to_json().unwrap());
        }
        // FIXME not sure what to do with these yet.
        if name == Some("raw".to_string()) {
            log::trace!("{}", self.to_json().unwrap());
        }
        package_url.host = name.unwrap_or("".to_string());

        package_url.version = self.get_version();
        if package_url.version.is_none() {
            package_url.version = self.main_derivation.get_version();
        }
        if package_url.version.is_none() {
            log::trace!("{}", self.to_json().unwrap());
        }

        // FIXME this cannot use the nix scope, which does not actually exist.
        // See https://github.com/package-url/purl-spec/blob/master/PURL-TYPES.rst
        // for the accepted scopes.
        package_url.scheme = "generic".to_string();

        let urls = self.main_derivation.get_urls();
        let url = match urls.get(0) {
            Some(u) => u,
            None => {
                log::trace!("{}", self.to_json().unwrap());
                return package_url;
            }
        };

        // TODO detect the scheme using the url.
        if url.starts_with("https://crates.io") {
            package_url.scheme = "cargo".to_string();
        }
        if url.starts_with("https://www.cpan.org/") {
            package_url.scheme = "cpan".to_string();
        }
        if url.starts_with("https://rubygems.org") {
            package_url.scheme = "gem".to_string();
        }
        if url.starts_with("https://hackage.haskell.org/") {
            package_url.scheme = "hackage".to_string();
        }
        if url.starts_with("https://repo.maven.apache.org/maven2") {
            package_url.scheme = "maven".to_string();
        }
        if url.starts_with("https://registry.npmjs.org") {
            package_url.scheme = "npm".to_string();
        }
        if url.starts_with("https://www.nuget.org") {
            package_url.scheme = "nuget".to_string();
        }
        if url.starts_with("https://bitbucket.org") {
            package_url.scheme = "bitbucket".to_string();
        }
        if url.starts_with("https://hub.docker.com") {
            package_url.scheme = "docker".to_string();
        }
        if url.starts_with("https://pypi.org") || url.starts_with("https://pypi.python.org") {
            package_url.scheme = "pypi".to_string();
        }
        // if url.starts_with("https://github.com") {
        //     package_url.scheme = "gem".to_string();
        // }
        // if url.starts_with("https://crates.io") {}
        // https://crates.io/api/v1/crates/project-name/1.0.2/download
        // if url.starts_with("https://bitbucket.org") {}
        // if url.starts_with("https://registry.npmjs.org") {}
        // if url.starts_with("https://pypi.python.org") {}
        // if url.starts_with("https://github.com") {}
        // TODO How can we detect go and swift packages? The url will just be another git URL
        // TODO gitlab ??
        // TODO openwrt ??

        // According to the PURL doc, for the generic scope:
        // > There is no default repository. A download_url and checksum may be provided in qualifiers
        // > or as separate attributes outside of a purl for proper identification and location.
        // https://github.com/package-url/purl-spec/blob/346589846130317464b677bc4eab30bf5040183a/PURL-TYPES.rst#generic
        package_url
            .query_params
            .insert("download_url".to_string(), url.to_string());
        // Format should be sha256:de4d501267da...
        // package_url
        //     .query_params
        //     .insert("checksum".to_string(), url.to_string());
        return package_url;
    }

    pub fn to_json(&self) -> Result<String, String> {
        return serde_json::to_string_pretty(self.clone()).map_err(|e| e.to_string());
    }

    pub fn print_out_paths(&self, package_graph: &PackageGraph, depth: usize) -> String {
        if is_stdenv(&self.main_derivation.get_name().unwrap_or("".to_string())) {
            return "".to_string();
        }

        let mut response = "".to_string();
        for child_derivation_path in self.children.iter() {
            let child_derivation = package_graph.nodes.get(child_derivation_path).unwrap();

            let out_path = "  ".repeat(depth) + &child_derivation_path + "\n";
            response += &out_path;
            response += &child_derivation.print_out_paths(package_graph, depth + 1);
        }
        response
    }

    pub fn pretty_print(
        &self,
        graph: &PackageGraph,
        depth: usize,
        display_options: &DisplayOptions,
    ) -> Vec<PrettyPrintLine> {
        let mut lines: Vec<PrettyPrintLine> = vec![];

        // FIXME this should be configurable.
        if self.is_inline_script() {
            return lines;
        }

        if depth >= display_options.max_depth.unwrap_or(std::usize::MAX) {
            return lines;
        }

        lines.push(PrettyPrintLine::new(self.get_purl().to_string(), depth));

        if !display_options.print_only_purl {
            if let Some(p) = &self.package {
                for line in p.pretty_print(depth, display_options) {
                    lines.push(line);
                }
            }
            for line in self.main_derivation.pretty_print(depth, display_options) {
                lines.push(line);
            }
            if self.sources.len() != 0 {
                lines.push(PrettyPrintLine::new("sources:", depth + 1));
                for source in &self.sources {
                    for line in source.pretty_print(depth + 1, display_options) {
                        lines.push(line);
                    }
                }
            }
            if self.patches.len() != 0 {
                lines.push(PrettyPrintLine::new("patches:", depth + 1));
                for patch_path in &self.patches {
                    let patch = &graph.nodes.get(patch_path).unwrap().main_derivation;
                    for line in patch.pretty_print(depth + 1, display_options) {
                        lines.push(line);
                    }
                }
            }
        }

        if self.children.len() != 0 {
            for child_package_derivation_path in self.children.iter() {
                let child_package = match graph.nodes.get(child_package_derivation_path) {
                    Some(p) => p,
                    None => {
                        log::warn!(
                            "Could not get package in package graph for {}",
                            &child_package_derivation_path
                        );
                        continue;
                    }
                };
                if !display_options.print_stdenv
                    && is_stdenv(&child_package.main_derivation.get_name().unwrap())
                {
                    continue;
                }

                for line in child_package.pretty_print(&graph, depth + 1, display_options) {
                    lines.push(line);
                }
            }
        }
        lines
    }
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(PartialEq)]
pub struct PackageGraphStats {
    pub nodes_count: usize,

    /// Number of nodes that are reachable from the root nodes.
    pub reachable_nodes_count: BTreeMap<String, usize>,

    /// The length of the longest path from the root nodes to a leaf node.
    pub longest_path_length: BTreeMap<String, usize>,

    pub root_nodes_count: usize,

    pub patches_count: usize,

    /// Number of derivations which had an associated entry in the package meta dictionnary.
    pub package_meta_count: usize,

    pub purl_scope_count: BTreeMap<String, usize>,
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(PartialEq)]
pub struct PackageGraph {
    pub nodes: BTreeMap<String, PackageNode>,
    pub root_nodes: BTreeSet<String>,
}

impl PackageGraph {
    pub fn get_stats(&self) -> PackageGraphStats {
        let mut package_graph_stats = PackageGraphStats::default();
        package_graph_stats.nodes_count = self.nodes.len();
        package_graph_stats.root_nodes_count = self.root_nodes.len();
        for root_node in &self.root_nodes {
            let package_node = self.nodes.get(root_node).unwrap();
            package_graph_stats.reachable_nodes_count.insert(
                root_node.clone(),
                package_node.get_reachable_nodes_count(&self.nodes, &mut HashSet::default()),
            );
            package_graph_stats.longest_path_length.insert(
                root_node.clone(),
                package_node
                    .get_longest_path(&root_node, &self.nodes, &mut HashMap::default())
                    .len(),
            );
            package_graph_stats.purl_scope_count = self.get_purl_scope_stats();
            let longest_path = package_node.get_longest_path(&root_node, &self.nodes, &mut HashMap::default());
        }
        package_graph_stats
    }

    pub fn get_purl_scope_stats(&self) -> BTreeMap<String, usize> {
        let mut visited_children: HashSet<String> = HashSet::default();

        let mut response: BTreeMap<String, usize> = BTreeMap::default();
        let mut node_queue = self.root_nodes.clone();

        while !node_queue.is_empty() {
            let current_node_path = node_queue.pop_first().unwrap();

            if visited_children.contains(&current_node_path) {
                continue;
            }

            let current_node = self.nodes.get(&current_node_path).unwrap();
            let purl = current_node.get_purl();

            if response.contains_key(&purl.scheme) {
                let count = response.get_mut(&purl.scheme).unwrap();
                *count += 1;
            } else {
                response.insert(purl.scheme.clone(), 1);
            }

            // FIXME we should also go through the patches?
            for current_node_child in &current_node.children {
                node_queue.insert(current_node_child.clone());
            }
            visited_children.insert(current_node_path.clone());
        }

        response
    }

    pub fn print_out_paths(&self) -> String {
        let mut response: String = "".to_string();
        for derivation_path in &self.root_nodes {
            let child_derivation = self.nodes.get(derivation_path).unwrap();
            let out_path = "  ".repeat(0) + &derivation_path + "\n";
            response += &out_path;
            let child_derivation = self.nodes.get(derivation_path).unwrap();
            response += &child_derivation.print_out_paths(self, 1);
        }
        response
    }

    pub fn pretty_print(&self, depth: usize, display_options: &DisplayOptions) -> String {
        let mut lines: Vec<PrettyPrintLine> = vec![];
        let mut response = "".to_string();

        let mut visited_children: HashSet<String> = HashSet::default();
        for (derivation_path, package_node) in &self.nodes {
            if visited_children.contains(derivation_path) {
                continue;
            }
            for child_derivation_path in &package_node.children {
                let child = self.nodes.get(child_derivation_path).unwrap().clone();
                add_visited_children(child, &self, &mut visited_children);
            }
        }

        for (derivation_path, package_node) in &self.nodes {
            if !display_options.print_stdenv && is_stdenv(&package_node.main_derivation.get_name().unwrap()) {
                continue;
            }
            for line in package_node.pretty_print(self, depth, display_options) {
                lines.push(line);
            }
        }

        for line in lines {
            response += &line.to_string();
            response += "\n";
        }
        response
    }
}

fn add_visited_children(
    package_node: &PackageNode,
    package_graph: &PackageGraph,
    visited_children: &mut HashSet<String>,
) {
    for child_derivation_path in &package_node.children {
        if visited_children.contains(child_derivation_path) {
            continue;
        }
        visited_children.insert(child_derivation_path.to_string());
        let child_package = match package_graph.nodes.get(child_derivation_path) {
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

// Small struct to make it easier to pretty-print the
// internal representation for the package graph.
#[derive(Debug)]
pub struct PrettyPrintLine {
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
        let package = match packages.get(&derivation_name) {
            Some(p) => Some(p.clone()),
            None => None,
        };
        let mut current_node = PackageNode {
            package,
            main_derivation: derivation.clone(),
            children: BTreeSet::default(),
            sources: vec![],
            patches: BTreeSet::default(),
        };
        let current_node_patches = derivation.get_patches();

        let mut child_derivation_paths: BTreeSet<String> = BTreeSet::default();
        for input_derivation_path in derivation.input_derivations.keys() {
            child_derivation_paths.insert(input_derivation_path.clone());
        }

        let mut visited_derivations: HashSet<String> = HashSet::default();

        while child_derivation_paths.len() != 0 {
            let child_derivation_path = child_derivation_paths.pop_last().unwrap();
            log::debug!("Visiting {}", &child_derivation_path);
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
                    "NOT_AN_ACTUAL_NAME".to_string()
                }
            };
            if child_derivation_name != "source" && packages.get(&child_derivation_name).is_some() {
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
                    current_node.patches.insert(child_derivation_path.clone());
                } else {
                    current_node.sources.push(child_derivation.clone());
                }
            }

            for input_derivation_path in child_derivation.input_derivations.keys() {
                child_derivation_paths.insert(input_derivation_path.clone());
            }
        }
        response.nodes.insert(derivation_path.clone(), current_node);
    }
    response
}

pub fn get_package_graph_next(
    derivations: &crate::nix::Derivations,
    packages: &crate::nix::Packages,
) -> PackageGraph {
    let mut response = PackageGraph::default();

    let mut all_child_derivations: HashSet<String> = HashSet::default();
    for (derivation_path, derivation) in derivations.iter() {
        let mut current_node = PackageNode {
            package: None,
            main_derivation: derivation.clone(),
            children: BTreeSet::default(),
            sources: vec![],
            patches: BTreeSet::default(),
        };

        let current_node_patches = derivation.get_patches();

        for input_derivation_path in derivation.input_derivations.keys() {
            let child_derivation = derivations.get(input_derivation_path).unwrap();
            if let Some(child_derivation_out_path) = child_derivation.env.get("out") {
                if current_node_patches.contains(child_derivation_out_path) {
                    current_node.patches.insert(input_derivation_path.clone());
                    all_child_derivations.insert(input_derivation_path.clone());
                    continue;
                }
            }

            current_node.children.insert(input_derivation_path.to_string());
            all_child_derivations.insert(input_derivation_path.clone());
        }

        response.nodes.insert(derivation_path.clone(), current_node);
    }

    for (derivation_path, derivation) in derivations.iter() {
        if all_child_derivations.contains(derivation_path) {
            continue;
        }
        response.root_nodes.insert(derivation_path.to_string());
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

    #[test]
    pub fn parse_package_metadata_malformed_maintainers() {
        const package_metadata: &str = r###"
          {
            "meta": {
              "available": false,
              "broken": false,
              "description": "Software for rapid LiDAR processing",
              "homepage": "http://lastools.org/",
              "insecure": false,
              "license": {
                "deprecated": false,
                "free": false,
                "fullName": "Unfree",
                "redistributable": false,
                "shortName": "unfree"
              },
              "maintainers": [
                {
                  "github": "StephenWithPH",
                  "githubId": 2990492,
                  "name": "StephenWithPH"
                }
              ],
              "name": "LAStools-2.0.2",
              "outputsToInstall": [
                "out"
              ],
              "platforms": [
                "i686-cygwin",
                "x86_64-cygwin",
                "x86_64-darwin",
                "i686-darwin",
                "aarch64-darwin",
                "armv7a-darwin",
                "i686-freebsd13",
                "x86_64-freebsd13",
                "x86_64-solaris",
                "aarch64-linux",
                "armv5tel-linux",
                "armv6l-linux",
                "armv7a-linux",
                "armv7l-linux",
                "i686-linux",
                "loongarch64-linux",
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
                "x86_64-linux",
                "aarch64-netbsd",
                "armv6l-netbsd",
                "armv7a-netbsd",
                "armv7l-netbsd",
                "i686-netbsd",
                "m68k-netbsd",
                "mipsel-netbsd",
                "powerpc-netbsd",
                "riscv32-netbsd",
                "riscv64-netbsd",
                "x86_64-netbsd",
                "i686-openbsd",
                "x86_64-openbsd",
                "x86_64-redox"
              ],
              "unfree": true,
              "unsupported": false
            },
            "name": "LAStools-2.0.2",
            "outputName": "out",
            "outputs": {
              "out": null
            },
            "pname": "LAStools",
            "system": "x86_64-linux",
            "version": "2.0.2"
          }
        "###;
        let package: Package = serde_json::from_str(package_metadata).unwrap();
        assert_eq!(package.name, "LAStools-2.0.2");
    }

    #[test]
    pub fn get_version_from_rev() {
        let derivation: &str = r###"
          {
            "args": [
              "-e",
              "/nix/store/v7wqh83pzn39kjx6pdfixwyqlbmsqid3-builder.sh"
            ],
            "builder": "/nix/store/0rwyq0j954a7143p0wzd4rhycny8i967-bash-5.2-p15/bin/bash",
            "env": {
              "GIT_SSL_CAINFO": "/nix/store/9hx76jndjw5881pb83ghvlq6k4aqagz4-nss-cacert-3.92/etc/ssl/certs/ca-bundle.crt",
              "__structuredAttrs": "",
              "buildInputs": "",
              "builder": "/nix/store/0rwyq0j954a7143p0wzd4rhycny8i967-bash-5.2-p15/bin/bash",
              "cmakeFlags": "",
              "configureFlags": "",
              "deepClone": "",
              "depsBuildBuild": "",
              "depsBuildBuildPropagated": "",
              "depsBuildTarget": "",
              "depsBuildTargetPropagated": "",
              "depsHostHost": "",
              "depsHostHostPropagated": "",
              "depsTargetTarget": "",
              "depsTargetTargetPropagated": "",
              "doCheck": "",
              "doInstallCheck": "",
              "fetchLFS": "",
              "fetchSubmodules": "1",
              "fetcher": "/nix/store/fqkqnkkwzhqn21fh9ba4nz75nhd89irm-nix-prefetch-git",
              "impureEnvVars": "http_proxy https_proxy ftp_proxy all_proxy no_proxy GIT_PROXY_COMMAND NIX_GIT_SSL_CAINFO SOCKS_SERVER",
              "leaveDotGit": "",
              "mesonFlags": "",
              "name": "source",
              "nativeBuildInputs": "/nix/store/zy5r5ssh2zk6n1k34gv09fd9865lcniq-git-minimal-2.40.1",
              "nonConeMode": "",
              "out": "/nix/store/gj39c9gmjz3z5f6lgkcsl0lc07fwhq0c-source",
              "outputHash": "sha256-I3PGgh0XqRkCFz7lUZ3Q4eU0+0GwaQcVb6t4Pru1kKo=",
              "outputHashMode": "recursive",
              "outputs": "out",
              "patches": "",
              "postFetch": "",
              "preferLocalBuild": "1",
              "propagatedBuildInputs": "",
              "propagatedNativeBuildInputs": "",
              "rev": "v0.8.2",
              "sparseCheckout": "",
              "stdenv": "/nix/store/9v8sc2q2dflxjcz1hsw84b10bvg0wand-stdenv-linux",
              "strictDeps": "",
              "system": "x86_64-linux",
              "url": "https://github.com/libjxl/libjxl.git"
            },
            "inputDrvs": {
              "/nix/store/331fppp0q0n5xy5mrhkg3abp3sbpb869-stdenv-linux.drv": [
                "out"
              ],
              "/nix/store/33xwn0p89b0iwqxqdnp36hyy138flhkg-nss-cacert-3.92.drv": [
                "out"
              ],
              "/nix/store/hla091y2jgs76hd8ps5ky6d81qzkdfz5-bash-5.2-p15.drv": [
                "out"
              ],
              "/nix/store/pz619bwk8qgpb0zd3g11fm0hclk3pfz3-git-minimal-2.40.1.drv": [
                "out"
              ]
            },
            "inputSrcs": [
              "/nix/store/fqkqnkkwzhqn21fh9ba4nz75nhd89irm-nix-prefetch-git",
              "/nix/store/v7wqh83pzn39kjx6pdfixwyqlbmsqid3-builder.sh"
            ],
            "outputs": {
              "out": {
                "hash": "2373c6821d17a91902173ee5519dd0e1e534fb41b06907156fab783ebbb590aa",
                "hashAlgo": "r:sha256",
                "path": "/nix/store/gj39c9gmjz3z5f6lgkcsl0lc07fwhq0c-source"
              }
            },
            "system": "x86_64-linux"
          }
        "###;
        let derivation: Derivation = serde_json::from_str(derivation).unwrap();
        assert_eq!(derivation.get_name(), Some("libjxl".to_string()));
        assert_eq!(derivation.get_version(), Some("0.8.2".to_string()));
    }

    #[test]
    pub fn test_remove_version_from_name() {
        let derivation: &str = r###"
          {
            "args": [
              "-e",
              "/nix/store/6xg259477c90a229xwmb53pdfkn6ig3g-default-builder.sh"
            ],
            "builder": "/nix/store/0rwyq0j954a7143p0wzd4rhycny8i967-bash-5.2-p15/bin/bash",
            "env": {
              "LDFLAGS": "",
              "__structuredAttrs": "",
              "bin": "/nix/store/j41ms763gpyya3hylqmaq1p108bhvkcm-zstd-1.5.5-bin",
              "buildInputs": "/nix/store/2i83qvxdxps1s91335icgkd2mp6v6b91-bash-5.2-p15-dev",
              "builder": "/nix/store/0rwyq0j954a7143p0wzd4rhycny8i967-bash-5.2-p15/bin/bash",
              "checkPhase": "runHook preCheck\n# Patch shebangs for playTests\npatchShebangs ../programs/zstdgrep\nctest -R playTests # The only relatively fast test.\nrunHook postCheck\n",
              "cmakeDir": "../build/cmake",
              "cmakeFlags": "-DZSTD_BUILD_CONTRIB:BOOL=ON -DZSTD_BUILD_SHARED:BOOL=ON -DZSTD_BUILD_STATIC:BOOL=OFF -DZSTD_BUILD_TESTS:BOOL=ON -DZSTD_LEGACY_SUPPORT:BOOL=OFF -DZSTD_PROGRAMS_LINK_SHARED:BOOL=ON",
              "configureFlags": "",
              "depsBuildBuild": "",
              "depsBuildBuildPropagated": "",
              "depsBuildTarget": "",
              "depsBuildTargetPropagated": "",
              "depsHostHost": "",
              "depsHostHostPropagated": "",
              "depsTargetTarget": "",
              "depsTargetTargetPropagated": "",
              "dev": "/nix/store/4pifi04nz9l2i0l692ny148q94klml0r-zstd-1.5.5-dev",
              "doCheck": "1",
              "doInstallCheck": "",
              "dontUseCmakeBuildDir": "1",
              "man": "/nix/store/qxv3dnwvi2xw1kx8bhf8lcyssbdvna8d-zstd-1.5.5-man",
              "mesonFlags": "",
              "name": "zstd-1.5.5",
              "nativeBuildInputs": "/nix/store/hdwhs75n9ydc4pdqv0hamjzjv1fkw1zz-cmake-boot-3.25.3 /nix/store/xv00ljxfrgdi9m53w96mj7pqgb0m0c3l-file-5.44-dev",
              "out": "/nix/store/81d38brw9cnw2qk2kynrf5dr6hhkcq66-zstd-1.5.5",
              "outputs": "bin dev man out",
              "patches": "/nix/store/n91acyjrlchm0snw0w16i4683pf788ax-playtests-darwin.patch",
              "pname": "zstd",
              "postPatch": "substituteInPlace build/cmake/CMakeLists.txt \\\n  --replace 'message(SEND_ERROR \"You need to build static library to build tests\")' \"\"\nsubstituteInPlace build/cmake/tests/CMakeLists.txt \\\n  --replace 'libzstd_static' 'libzstd_shared'\nsed -i \\\n  \"1aexport LD_LIBRARY_PATH=$PWD/build_/lib\" \\\n  tests/playTests.sh\n",
              "preConfigure": "mkdir -p build_ && cd $_\n",
              "preInstall": "mkdir -p $bin/bin\nsubstituteInPlace ../programs/zstdgrep \\\n  --replace \":-grep\" \":-/nix/store/b4in4hmq54h6l34a0v6ha40z97c0lzw2-gnugrep-3.7/bin/grep\" \\\n  --replace \":-zstdcat\" \":-$bin/bin/zstdcat\"\n\nsubstituteInPlace ../programs/zstdless \\\n  --replace \"zstdcat\" \"$bin/bin/zstdcat\"\ncp contrib/pzstd/pzstd $bin/bin/pzstd\n",
              "propagatedBuildInputs": "",
              "propagatedNativeBuildInputs": "",
              "src": "/nix/store/2w0cnsrfgapi5jf9z9yciir4hgz7nyj8-source",
              "stdenv": "/nix/store/gv2cl6qvvslz5h15vqd89f1rpvrdg5yc-stdenv-linux",
              "strictDeps": "",
              "system": "x86_64-linux",
              "version": "1.5.5"
            },
            "inputDrvs": {
              "/nix/store/975cwk57d5xy6cyakapsifyg19n3g516-file-5.44.drv": [
                "dev"
              ],
              "/nix/store/f69xwgpf415s7fvg64qhsfmqpmb7xnjg-source.drv": [
                "out"
              ],
              "/nix/store/hla091y2jgs76hd8ps5ky6d81qzkdfz5-bash-5.2-p15.drv": [
                "dev",
                "out"
              ],
              "/nix/store/im7dywhi0ycfnkdplpmh9xqzynf6v2mg-cmake-boot-3.25.3.drv": [
                "out"
              ],
              "/nix/store/m4cyqwyzda46912dirznjzx5cml6d018-gnugrep-3.7.drv": [
                "out"
              ],
              "/nix/store/z9vnfwzs0226f7qid0j0iglfbpvb61hx-stdenv-linux.drv": [
                "out"
              ]
            },
            "inputSrcs": [
              "/nix/store/6xg259477c90a229xwmb53pdfkn6ig3g-default-builder.sh",
              "/nix/store/n91acyjrlchm0snw0w16i4683pf788ax-playtests-darwin.patch"
            ],
            "outputs": {
              "bin": {
                "path": "/nix/store/j41ms763gpyya3hylqmaq1p108bhvkcm-zstd-1.5.5-bin"
              },
              "dev": {
                "path": "/nix/store/4pifi04nz9l2i0l692ny148q94klml0r-zstd-1.5.5-dev"
              },
              "man": {
                "path": "/nix/store/qxv3dnwvi2xw1kx8bhf8lcyssbdvna8d-zstd-1.5.5-man"
              },
              "out": {
                "path": "/nix/store/81d38brw9cnw2qk2kynrf5dr6hhkcq66-zstd-1.5.5"
              }
            },
            "system": "x86_64-linux"
          }
        "###;
        let derivation: Derivation = serde_json::from_str(derivation).unwrap();
        assert_eq!(derivation.get_name(), Some("zstd".to_string()));
        assert_eq!(derivation.get_version(), Some("1.5.5".to_string()));
    }

    #[test]
    pub fn test_get_name_from_pname() {
        let derivation: &str = r###"
          {
            "args": [
              "-e",
              "/nix/store/6xg259477c90a229xwmb53pdfkn6ig3g-default-builder.sh"
            ],
            "builder": "/nix/store/wllx077cz9z34zgrhwj2fc8r5r1hn6mx-bash-5.2-p15/bin/bash",
            "env": {
              "LANG": "C.UTF-8",
              "__structuredAttrs": "",
              "buildInputs": "/nix/store/3806m4syrck3andi5bw1jygclzrkbqnw-cairo-1.16.0-dev",
              "builder": "/nix/store/wllx077cz9z34zgrhwj2fc8r5r1hn6mx-bash-5.2-p15/bin/bash",
              "cmakeFlags": "",
              "configureFlags": "",
              "depsBuildBuild": "",
              "depsBuildBuildPropagated": "",
              "depsBuildTarget": "",
              "depsBuildTargetPropagated": "",
              "depsHostHost": "",
              "depsHostHostPropagated": "",
              "depsTargetTarget": "",
              "depsTargetTargetPropagated": "",
              "disallowedReferences": "",
              "doCheck": "",
              "doInstallCheck": "1",
              "mesonFlags": "-Dpython=/nix/store/bhnfk4sy0ki5kp7hrbcq7pyqppbg7cwv-python3-3.10.13/bin/python3.10",
              "name": "python3.10-pycairo-1.23.0",
              "nativeBuildInputs": "/nix/store/bhnfk4sy0ki5kp7hrbcq7pyqppbg7cwv-python3-3.10.13 /nix/store/pg4wl9d9c6wb9i50wrgzxp5iqb0s5vbf-wrap-python-hook /nix/store/2znsgzazszhd9jns39fc688c15sdqakw-ensure-newer-sources-hook /nix/store/6ljpqq1imwfxadh586lj33ck7s84cgzv-python-remove-tests-dir-hook /nix/store/zq4g4h4dqvlk72nr9plcrqacxl37z3s1-python-catch-conflicts-hook /nix/store/3srpdh3b7an72xjnsdcl0h1ybpqwfq40-python-remove-bin-bytecode-hook /nix/store/vhns06ximyi3r11qy4zv3f63vrlg7kss-python-imports-check-hook.sh /nix/store/gzzh23js0w8mjr578j6sy7zlw6qlnzqz-python-namespaces-hook.sh /nix/store/aqjdg2d3g7nd4p0lwgr13v66vap3w9kd-meson-1.1.0 /nix/store/g0m0q186glhbxzv2mbh6y201r0gw989x-ninja-1.11.1 /nix/store/7g8wjkxx774bbnx0v06xc58qap2pvj6l-pkg-config-wrapper-0.29.2 /nix/store/459n6kmy4hsdaq8lr8s7ap7bshzymv2w-pytest-check-hook",
              "out": "/nix/store/cp5adghz8pyv01a15w1a8xxzmhqrv7nj-python3.10-pycairo-1.23.0",
              "outputs": "out",
              "patches": "",
              "pname": "pycairo",
              "postFixup": "wrapPythonPrograms\n",
              "propagatedBuildInputs": "/nix/store/bhnfk4sy0ki5kp7hrbcq7pyqppbg7cwv-python3-3.10.13",
              "propagatedNativeBuildInputs": "",
              "src": "/nix/store/ai0y1sqcklhjzjg04s7js5xc92g9j542-source",
              "stdenv": "/nix/store/zpcxdbzssnhig0czrfvc1bd33lzrdy2i-stdenv-linux",
              "strictDeps": "1",
              "system": "i686-linux",
              "version": "1.23.0"
            },
            "inputDrvs": {
              "/nix/store/098ldl2py7yz31vwry8nhcjwrvs8gp56-source.drv": [
                "out"
              ],
              "/nix/store/3mrp1f80d1838n6wrmcn2w8dbzd1b53d-python-remove-tests-dir-hook.drv": [
                "out"
              ],
              "/nix/store/7aqhla8b73157glwx8klvnvdq014ykri-python-namespaces-hook.sh.drv": [
                "out"
              ],
              "/nix/store/9m6yhn4w3myqhk6w83xfkl81w7d71avl-stdenv-linux.drv": [
                "out"
              ],
              "/nix/store/ampp49cj5qvql9wlxarjpfybpqvn12nm-pytest-check-hook.drv": [
                "out"
              ],
              "/nix/store/drqc6nfs5jg0cq6a65smcml6zysvbrnp-cairo-1.16.0.drv": [
                "dev"
              ],
              "/nix/store/h4nfr8plqsc7628xgngqvc3ddr1kbj65-ensure-newer-sources-hook.drv": [
                "out"
              ],
              "/nix/store/lj7902rl4qzxg8aa60y49m802759nh1p-meson-1.1.0.drv": [
                "out"
              ],
              "/nix/store/mp10xqzg2ww4v0i7v81ccz5k0x9im77q-bash-5.2-p15.drv": [
                "out"
              ],
              "/nix/store/njsalvap31p8vxyl22pzrdjxn34fb1kv-python3-3.10.13.drv": [
                "out"
              ],
              "/nix/store/q3lg2gv800gzhx123jgyc2y85zv7i15y-python-imports-check-hook.sh.drv": [
                "out"
              ],
              "/nix/store/rmi4ax1bzwdn0kgy2pr0yxrkcakyaw2d-pkg-config-wrapper-0.29.2.drv": [
                "out"
              ],
              "/nix/store/s6da7mav0ac0fmqc4f5vrbh3qlfhgyx7-wrap-python-hook.drv": [
                "out"
              ],
              "/nix/store/v8hm8h7cz5jj191fx071m18l04mp2vbx-ninja-1.11.1.drv": [
                "out"
              ],
              "/nix/store/x28mrjkg28i9srjyhw1249f3g42hzyj4-python-catch-conflicts-hook.drv": [
                "out"
              ],
              "/nix/store/y7qkcyqqlgy24hc42vfzildzhx5nx9pi-python-remove-bin-bytecode-hook.drv": [
                "out"
              ]
            },
            "inputSrcs": [
              "/nix/store/6xg259477c90a229xwmb53pdfkn6ig3g-default-builder.sh"
            ],
            "outputs": {
              "out": {
                "path": "/nix/store/cp5adghz8pyv01a15w1a8xxzmhqrv7nj-python3.10-pycairo-1.23.0"
              }
            },
            "system": "i686-linux"

          }
        "###;
        let derivation: Derivation = serde_json::from_str(derivation).unwrap();
        assert_eq!(derivation.get_name(), Some("pycairo".to_string()));
        assert_eq!(derivation.get_version(), Some("1.23.0".to_string()));
    }
}
