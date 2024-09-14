use serde_spdx::spdx::v_2_3::{
    SpdxBuilder, SpdxCreationInfoBuilder, SpdxItemPackages, SpdxItemPackagesBuilder,
};

pub fn dump(
    package_graph: &crate::nix::PackageGraph,
    _format: &crate::sbom::SerializationFormat,
    options: &crate::nix::DumpOptions,
) -> Result<String, anyhow::Error> {
    let creation_info = SpdxCreationInfoBuilder::default()
        .created("created")
        .creators(vec![])
        .build()?;
    let root_node_id = match package_graph.get_root_node() {
        Some(n) => n,
        None => return Ok("Expected to find a single root node when dumping to sdpx format".to_string()),
    };
    let root_derivation = package_graph.nodes.get(&root_node_id).unwrap();

    let mut spdx_builder = SpdxBuilder::default();

    // Generate a new uuid for this manifest
    let uuid = uuid::Uuid::new_v4();
    let name = root_derivation.id.clone();

    let spdx_builder = spdx_builder
        .creation_info(creation_info)
        .packages(vec![])
        // DISCUSS Should the document namespace be something like the path of the root derivation?
        // This would make the namespace content-addressed, and thus allow other SPDX documents
        // to reference this one.
        // .document_namespace()
        .relationships(vec![])
        .data_license("temp")
        .document_namespace(format!("https://spdx.org/spdxdocs/{}-{}", name, uuid))
        // .files(files)
        .spdx_version("SPDX-2.3")
        .spdxid("SPDXRef-DOCUMENT")
        .name(name.clone());

    let mut packages = vec![];
    for (_package_id, package) in &package_graph.nodes {
        let spdx_package = dump_package(package, &crate::nix::DumpOptions::default())?;
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
    let package_name = match package_node.get_name() {
        Some(n) => n,
        None => package_node.id.clone(),
    };

    let mut package_builder = SpdxItemPackagesBuilder::default();
    let package_builder = package_builder
        .name(package_name)
        .spdxid(package_node.id.clone())
        // .version_info(package_node.get_version().into())
        // .package_file_name("package_file_name")
        // .supplier("supplier")
        // .originator("originator")
        .files_analyzed(true)
        // .package_verification_code("verification_code")
        // .checksums("checksum")
        .homepage("home_page")
        .source_info("source_info")
        .license_concluded("license_concluded")
        .license_info_from_files(vec![])
        .license_declared("license_declared")
        .license_comments("license_comments")
        // .license_set("license_set")
        // .license_info_from_files("license_info_in_files")
        .license_declared("license_declared")
        .license_concluded("license_concluded")
        .license_info_from_files(vec![])
        .license_declared("license_declared")
        .license_comments("license_comments")
        // .license_set("license_set")
        // .license_info_from_files("license_info_in_files")
        .license_declared("license_declared")
        .license_concluded("license_concluded")
        .license_info_from_files(vec![])
        .license_declared("license_declared")
        .license_comments("license_comments");

    if let Some(package_version) = package_node.get_version() {
        package_builder.version_info(package_version);
    }

    if let Some(url) = &package_node.url {
        package_builder.download_location(url);
    } else {
        // FIXME this is obviously not what we should be doing.
        package_builder.download_location("https://example.com/archive-3.2.2.tar.gz");
    }

    Ok(package_builder.build()?)
}
