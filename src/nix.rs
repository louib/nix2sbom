use std::io::Error;
use std::process::Command;

pub fn get_derivation(file: &str) -> Result<Vec<u8>, Error> {
    let output = Command::new("nix")
        .arg("show-derivation")
        .arg("-r")
        .arg("-f")
        .arg(file)
        .output()?;

    Ok(output.stdout)
}
