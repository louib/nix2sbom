pub const CYCLONE_DX_NAME: &str = "CycloneDX";
pub const SPDX_NAME: &str = "SPDX";
pub const PRETTY_PRINT_NAME: &str = "pretty-print";
pub const OUT_PATHS_NAME: &str = "pretty-print";

pub enum Format {
    SPDX,
    CycloneDX,
    PrettyPrint,
    OutPaths,
}

impl Format {
    pub fn from_string(format: &str) -> Option<Format> {
        if format.ends_with("spdx") {
            return Some(Format::SPDX);
        }
        if format.ends_with("cdx") {
            return Some(Format::CycloneDX);
        }
        if format.ends_with("pretty") {
            return Some(Format::PrettyPrint);
        }
        if format.ends_with("out-paths") {
            return Some(Format::OutPaths);
        }
        None
    }

    pub fn to_pretty_name(&self) -> String {
        match self {
            crate::sbom::Format::CycloneDX => CYCLONE_DX_NAME.to_string(),
            crate::sbom::Format::SPDX => SPDX_NAME.to_string(),
            crate::sbom::Format::PrettyPrint => PRETTY_PRINT_NAME.to_string(),
            crate::sbom::Format::OutPaths => OUT_PATHS_NAME.to_string(),
        }
    }

    pub fn get_default_serialization_format(&self) -> SerializationFormat {
        match self {
            crate::sbom::Format::CycloneDX => crate::sbom::SerializationFormat::JSON,
            crate::sbom::Format::SPDX => crate::sbom::SerializationFormat::JSON,
            // We don't really care which value is returned in those cases.
            crate::sbom::Format::PrettyPrint => crate::sbom::SerializationFormat::XML,
            crate::sbom::Format::OutPaths => crate::sbom::SerializationFormat::XML,
        }
    }

    pub fn dump(
        &self,
        serialization_format: &SerializationFormat,
        package_graph: &crate::nix::PackageGraph,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            crate::sbom::Format::CycloneDX => {
                return match crate::cyclone_dx::dump(&package_graph, &serialization_format) {
                    Ok(d) => Ok(d),
                    Err(s) => Err(Box::new(crate::errors::Error::UnknownError(s))),
                };
            }
            crate::sbom::Format::SPDX => Err(Box::new(crate::errors::Error::UnsupportedFormat(
                "spdx".to_string(),
            ))),
            crate::sbom::Format::PrettyPrint => {
                let display_options = crate::nix::DisplayOptions {
                    print_stdenv: false,
                    print_only_purl: true,
                    print_exclude_list: vec![],
                    max_depth: Some(1),
                };

                return Ok(crate::nix::pretty_print_package_graph(
                    &package_graph,
                    0,
                    &display_options,
                ));
            }
            crate::sbom::Format::OutPaths => {
                return Ok(crate::nix::print_out_paths(&package_graph));
            }
        }
    }
}

impl Default for Format {
    fn default() -> Format {
        Format::CycloneDX
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum SerializationFormat {
    JSON,
    YAML,
    XML,
}

impl SerializationFormat {
    pub fn from_string(format: &str) -> Option<SerializationFormat> {
        if format.ends_with("json") {
            return Some(SerializationFormat::JSON);
        }
        if format.ends_with("yaml") || format.ends_with("yml") {
            return Some(SerializationFormat::YAML);
        }
        if format.ends_with("xml") {
            return Some(SerializationFormat::XML);
        }
        None
    }
}
