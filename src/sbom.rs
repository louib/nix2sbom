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
