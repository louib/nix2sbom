pub enum Format {
    SPDX,
    CycloneDX,
}

impl Format {
    pub fn from_string(format: &str) -> Option<Format> {
        if format.ends_with("spdx") {
            return Some(Format::SPDX);
        }
        if format.ends_with("cdx") {
            return Some(Format::CycloneDX);
        }
        None
    }

    pub fn to_pretty_name(&self) -> String {
        match self {
            crate::sbom::Format::CycloneDX => "CycloneDX".to_string(),
            crate::sbom::Format::SPDX => "SPDX".to_string(),
        }
    }
}

impl Default for Format {
    fn default() -> Format {
        Format::CycloneDX
    }
}

enum SerializationFormat {
    JSON,
    YAML,
    XML,
}
