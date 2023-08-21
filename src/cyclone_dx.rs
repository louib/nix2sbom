use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{de::Deserialize, ser::Serialize};

use serde_cyclonedx::cyclonedx::v_1_4::{
    Component, ComponentBuilder, CycloneDxBuilder, ExternalReference, ExternalReferenceBuilder,
    Metadata, ToolBuilder,
};

const CURRENT_SPEC_VERSION: &str = "1.4";

pub fn dump(package_graph: &crate::nix::PackageGraph) -> String {
    let mut metadata = Metadata::default();
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    metadata.timestamp = Some(now.to_rfc3339());

    metadata.tools = Some(vec![ToolBuilder::default()
        .vendor("louib".to_string())
        .name(crate::consts::PROJECT_NAME.to_string())
        .version(env!("CARGO_PKG_VERSION"))
        .build()
        .unwrap()]);

    let mut components: Vec<Component> = vec![];
    for (derivation_path, package) in package_graph.iter() {
        if let Some(component) = dump_derivation(derivation_path, package) {
            components.push(component);
        }
    }

    let cyclonedx = CycloneDxBuilder::default()
        .bom_format(crate::sbom::CYCLONE_DX_NAME)
        .spec_version(CURRENT_SPEC_VERSION)
        .version(1)
        .metadata(metadata)
        .components(components)
        .build()
        .unwrap();

    serde_json::to_string_pretty(&cyclonedx).unwrap()
}

pub fn dump_derivation(
    derivation_path: &str,
    package_node: &crate::nix::PackageNode,
) -> Option<Component> {
    log::debug!("Dumping derivation for {}", &derivation_path);
    let mut component_builder = ComponentBuilder::default();

    component_builder.bom_ref(derivation_path.to_string());
    component_builder.name(package_node.package.name.to_string());
    // component_builder.cpe("TODO".to_string())
    // TODO application is the generic type, but we should also use file and library
    // also, populate the mime_type in case of a file type.
    component_builder.type_("application".to_string());
    // I'm assuming here that if a package has been installed by Nix, it was required.
    component_builder.scope("required".to_string());
    component_builder.purl(package_node.package.get_purl());
    component_builder.version(package_node.package.version.to_string());

    if let Some(description) = &package_node.package.meta.description {
        component_builder.description(description.to_string());
    }

    if let Some(maintainers) = &package_node.package.meta.maintainers {
        let author = maintainers
            .iter()
            .map(|m| format!("{} ({})", m.name, m.email))
            .collect::<Vec<String>>()
            .join(" ");
        if author.len() != 0 {
            component_builder.author(author);
        }
    }

    let mut external_references: Vec<ExternalReference> = vec![];
    for homepage in package_node.package.meta.get_homepages() {
        // See https://docs.rs/serde-cyclonedx/latest/serde_cyclonedx/cyclonedx/v_1_5/struct.ExternalReference.html#structfield.type_
        // for all the available external reference types
        external_references.push(
            ExternalReferenceBuilder::default()
                .type_("website")
                .url(homepage.to_string())
                .build()
                .unwrap(),
        );
        if let Some(git_url) = crate::utils::get_git_url_from_generic_url(&homepage) {
            log::warn!("Found git url {} for homepage {}", &git_url, &homepage);
            external_references.push(
                ExternalReferenceBuilder::default()
                    .type_("vcs")
                    .url(git_url)
                    .build()
                    .unwrap(),
            );
        }
    }
    component_builder.external_references(external_references);

    Some(component_builder.build().unwrap())
}
