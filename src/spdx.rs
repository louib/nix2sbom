use chrono::Utc;
use serde_spdx::spdx::v_2_3::{
    SpdxBuilder, SpdxCreationInfoBuilder, SpdxItemPackages, SpdxItemPackagesBuilder,
};

// This is the only license accepted in the data_license field. See
// https://spdx.org/rdf/spdx-terms-v2.1/objectproperties/dataLicense___1140128580.html
// for details.
pub const CREATIVE_COMMONS_LICENSE: &str = "http://spdx.org/licenses/CC0-1.0";

pub fn dump(
    package_graph: &crate::nix::PackageGraph,
    _format: &crate::sbom::SerializationFormat,
    options: &crate::nix::DumpOptions,
) -> Result<String, anyhow::Error> {
    let creation_info = SpdxCreationInfoBuilder::default()
        // .created(&Utc::now().to_rfc3339())
        .created(&Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string())
        .creators(vec!["Tool: nix2sbom".to_string()])
        .build()?;
    let root_node_id = match package_graph.get_root_node() {
        Some(n) => n,
        None => return Ok("Expected to find a single root node when dumping to sdpx format".to_string()),
    };
    let root_package = package_graph.nodes.get(&root_node_id).unwrap();

    let mut spdx_builder = SpdxBuilder::default();

    // Generate a new uuid for this manifest
    let uuid = uuid::Uuid::new_v4();
    let name = root_package.id.clone();

    let spdx_builder = spdx_builder
        .creation_info(creation_info)
        .packages(vec![])
        // DISCUSS Should the document namespace be something like the path of the root derivation?
        // This would make the namespace content-addressed, and thus allow other SPDX documents
        // to reference this one.
        // .document_namespace()
        .document_namespace(format!("https://spdx.org/spdxdocs{}-{}", name, uuid))
        .relationships(vec![])
        .data_license(CREATIVE_COMMONS_LICENSE)
        .spdx_version("SPDX-2.3")
        .spdxid("SPDXRef-DOCUMENT")
        .name(name.clone());

    let mut packages = vec![];
    for (_package_id, package) in &package_graph.nodes_next {
        let spdx_package = dump_package(package, &options)?;
        packages.push(spdx_package);
    }

    spdx_builder.packages(packages);
    let spdx_manifest = spdx_builder.build()?;

    let response = match options.pretty {
        Some(false) => serde_json::to_string(&spdx_manifest)?,
        _ => serde_json::to_string_pretty(&spdx_manifest)?,
    };

    Ok(response)
}

fn dump_package(
    package_node: &crate::nix::PackageNode,
    _options: &crate::nix::DumpOptions,
) -> Result<SpdxItemPackages, anyhow::Error> {
    let package_name = match package_node.name.clone() {
        Some(n) => n,
        None => package_node.id.clone(),
    };

    let mut package_builder = SpdxItemPackagesBuilder::default();

    // The SPDX package identifier can only container letters, numbers,
    // and the characters `.` and `-`. This should probably be encapsulated
    // into a builder from the spdx crate.
    let spdx_id = format!("SPDXRef-{}", package_node.id.replace("/nix/store/", ""));
    let package_builder = package_builder.name(package_name).spdxid(spdx_id);

    if let Some(package_version) = package_node.version.clone() {
        package_builder.version_info(package_version);
    }

    if let Some(url) = &package_node.url {
        package_builder.download_location(url);
    } else {
        panic!(
            "No URL found for package {}. We will not include it in the manifest.",
            package_node.id
        );
    }

    let package = package_builder.build()?;
    Ok(package)
}
