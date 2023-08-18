use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{de::Deserialize, ser::Serialize};

use serde_cyclonedx::cyclonedx::v_1_4::{CycloneDxBuilder, Metadata, ToolBuilder};

pub fn dump() -> String {
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

    let cyclonedx = CycloneDxBuilder::default()
        .bom_format("CycloneDX")
        .spec_version("1.4")
        .version(1)
        .build()
        .unwrap();

    "".to_string()
}
