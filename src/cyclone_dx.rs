use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{de::Deserialize, ser::Serialize};

use serde_cyclonedx::cyclonedx::v_1_4::{
    Component, ComponentBuilder, CycloneDxBuilder, ExternalReference, ExternalReferenceBuilder,
    Metadata, ToolBuilder,
};

const CURRENT_SPEC_VERSION: &str = "1.4";

pub fn dump(derivations: &crate::nix::Derivations, packages: &crate::nix::Packages) -> String {
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
    for (derivation_path, derivation) in derivations.iter() {
        if let Some(component) = dump_derivation(derivation_path, derivation, packages) {
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
    derivation: &crate::nix::Derivation,
    packages: &crate::nix::Packages,
) -> Option<Component> {
    log::debug!("Dumping derivation for {}", &derivation_path);
    // TODO handle if the name was not found
    let derivation_name = match derivation.get_name() {
        Some(n) => n,
        None => return None,
    };

    log::info!("Getting package meta for derivation {}", derivation_name);
    let package = match crate::nix::get_package_for_derivation(derivation_name, packages) {
        Some(p) => p,
        None => {
            log::warn!("Could not find package meta for {}", derivation_name);
            return None;
        }
    };

    let mut component_builder = ComponentBuilder::default();

    component_builder.bom_ref(derivation_path.to_string());
    component_builder.name(package.name.to_string());
    // component_builder.cpe("TODO".to_string())
    // TODO application is the generic type, but we should also use file and library
    // also, populate the mime_type in case of a file type.
    component_builder.type_("application".to_string());
    // I'm assuming here that if a package has been installed by Nix, it was required.
    component_builder.scope("required".to_string());
    component_builder.purl(package.get_purl());
    component_builder.version(package.version.to_string());

    if let Some(description) = &package.meta.description {
        component_builder.description(description.to_string());
    }

    if let Some(maintainers) = &package.meta.maintainers {
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
    for homepage in package.meta.get_homepages() {
        // See https://docs.rs/serde-cyclonedx/latest/serde_cyclonedx/cyclonedx/v_1_5/struct.ExternalReference.html#structfield.type_
        // for all the available external reference types
        external_references.push(
            ExternalReferenceBuilder::default()
                .type_("website")
                .url(homepage)
                .build()
                .unwrap(),
        );
    }
    component_builder.external_references(external_references);

    Some(component_builder.build().unwrap())
}
