extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use std::io::Error;
use std::process::Command;

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
struct Derivation {
    outputs: HashMap<String, Output>,

    #[serde(rename = "inputSrcs")]
    inputs_sources: Vec<String>,

    #[serde(rename = "inputDrvs")]
    input_derivations: HashMap<String, Vec<String>>,

    system: String,

    builder: String,

    args: Vec<String>,

    env: HashMap<String, String>,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl Derivation {
    pub fn parse_from_file(file: &str) -> Result<Vec<u8>, Error> {
        let output = Command::new("nix")
            .arg("show-derivation")
            .arg("-r")
            .arg(file)
            .output()?;

        Ok(output.stdout)
    }
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
struct Output {
    path: String,
}

pub fn get_dependencies(path: &str) -> Vec<String> {
    // TODO nix-store -qR /an/executable/path
    vec![]
}
