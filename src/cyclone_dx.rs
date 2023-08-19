use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{de::Deserialize, ser::Serialize};

use serde_cyclonedx::cyclonedx::v_1_4::{
    Component, ComponentBuilder, CycloneDxBuilder, Metadata, ToolBuilder,
};

pub fn dump(derivations: &crate::nix::Derivations) -> String {
    let mut metadata = Metadata::default();
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    metadata.timestamp = Some(now.to_rfc3339());

    metadata.tools = Some(vec![ToolBuilder::default()
        .vendor("louib".to_string())
        .name("nix2sbom".to_string())
        .version(env!("CARGO_PKG_VERSION"))
        .build()
        .unwrap()]);

    let mut components: Vec<Component> = vec![];
    for derivation in derivations.values() {
        components.push(dump_derivation(derivation));
    }

    let cyclonedx = CycloneDxBuilder::default()
        .bom_format("CycloneDX")
        .spec_version("1.4")
        .version(1)
        .metadata(metadata)
        .components(components)
        .build()
        .unwrap();

    "".to_string()
}

pub fn dump_derivation(derivation: &crate::nix::Derivation) -> Component {
    ComponentBuilder::default()
        .name("TODO".to_string())
        // TODO application is the generic type, but we should also use file and library
        .type_("application".to_string())
        .version("TODO".to_string())
        .build()
        .unwrap()
}
