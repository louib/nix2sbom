use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[derive(PartialEq)]
struct NativePackage {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub purl: String,

    pub git_urls: Vec<String>,
    pub download_urls: Vec<String>,

    pub homepages: Vec<String>,

    pub source_derivation: String,
    // TODO add build derivations and input derivations
}

pub fn dump(
    package_graph: &crate::nix::PackageGraph,
    _format: &crate::format::SerializationFormat,
    options: &crate::nix::DumpOptions,
) -> Result<String, anyhow::Error> {
    let mut native_packages: Vec<NativePackage> = vec![];

    for package in package_graph.nodes.values() {
        let source_derivation = match &package.source_derivation {
            Some(derivation) => derivation,
            None => continue,
        };
        let native_package = NativePackage {
            id: package.id.clone(),
            // name: package.get_name(),
            name: Some("".to_string()),
            version: package.get_version(),
            purl: package.get_purl().to_string(),
            git_urls: package.get_git_urls(),
            download_urls: package.main_derivation.get_urls(),
            homepages: vec![],
            source_derivation: source_derivation.to_string(),
        };
        native_packages.push(native_package);
    }

    // Sort the native_packages by id
    native_packages.sort_by(|a, b| a.id.cmp(&b.id));

    let response = match options.pretty {
        Some(false) => serde_json::to_string(&native_packages)?,
        _ => serde_json::to_string_pretty(&native_packages)?,
    };

    Ok(response)
}
