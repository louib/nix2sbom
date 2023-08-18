enum Format {
    SPDX,
    CycloneDX,
}

impl Format {
    pub fn from_file_extension(file_path: &str) -> Option<Format> {
        if file_path.ends_with(".spdx") {
            return Some(Format::SPDX);
        }
        if file_path.ends_with(".cdx") {
            return Some(Format::CycloneDX);
        }
        None
    }
}

enum SerializationFormat {
    JSON,
    YAML,
    XML,
}
