pub mod cyclone_dx;
pub mod spdx;

pub const CYCLONE_DX_NAME: &str = "CycloneDX";
pub const SPDX_NAME: &str = "SPDX";
pub const PRETTY_PRINT_NAME: &str = "pretty-print";
pub const OUT_PATHS_NAME: &str = "pretty-print";
pub const STATS_NAME: &str = "stats";

pub enum Format {
    SPDX,
    CycloneDX,
    PrettyPrint,
    OutPaths,
    Stats,
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
        if format.ends_with("stats") {
            return Some(Format::Stats);
        }
        None
    }

    pub fn to_pretty_name(&self) -> String {
        match self {
            Format::CycloneDX => CYCLONE_DX_NAME.to_string(),
            Format::SPDX => SPDX_NAME.to_string(),
            Format::PrettyPrint => PRETTY_PRINT_NAME.to_string(),
            Format::OutPaths => OUT_PATHS_NAME.to_string(),
            Format::Stats => STATS_NAME.to_string(),
        }
    }

    pub fn get_default_serialization_format(&self) -> SerializationFormat {
        match self {
            Format::CycloneDX => SerializationFormat::JSON,
            Format::SPDX => SerializationFormat::JSON,
            Format::Stats => SerializationFormat::JSON,
            // We don't really care which value is returned in those cases.
            Format::PrettyPrint => SerializationFormat::XML,
            Format::OutPaths => SerializationFormat::XML,
        }
    }

    pub fn dump(
        &self,
        serialization_format: &SerializationFormat,
        package_graph: &crate::nix::PackageGraph,
        options: &crate::nix::DumpOptions,
    ) -> Result<String, anyhow::Error> {
        match self {
            Format::CycloneDX => {
                return match cyclone_dx::dump(&package_graph, &serialization_format, options) {
                    Ok(d) => Ok(d),
                    Err(s) => Err(anyhow::format_err!("Error dumping manifest: {}", s.to_string())),
                };
            }
            Format::SPDX => {
                return match spdx::dump(&package_graph, &serialization_format, options) {
                    Ok(d) => Ok(d),
                    Err(s) => Err(anyhow::format_err!("Error dumping manifest: {}", s.to_string())),
                };
            }
            Format::PrettyPrint => {
                let display_options = crate::nix::DisplayOptions {
                    print_stdenv: false,
                    print_only_purl: true,
                    print_exclude_list: vec![],
                    max_depth: Some(1),
                };

                return Ok(package_graph.pretty_print(0, &display_options));
            }
            Format::OutPaths => {
                return Ok(package_graph.print_out_paths());
            }
            Format::Stats => {
                return Ok(serde_json::to_string_pretty(&package_graph.get_stats(options))?);
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
    pub fn to_string(&self) -> String {
        match self {
            SerializationFormat::JSON => "json".to_string(),
            SerializationFormat::YAML => "yaml".to_string(),
            SerializationFormat::XML => "xml".to_string(),
        }
    }
}
