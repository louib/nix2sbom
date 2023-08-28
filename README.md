# nix2sbom
[![Build Status](https://github.com/louib/nix2sbom/actions/workflows/merge.yml/badge.svg?branch=main)](https://github.com/louib/nix2sbom/actions/workflows/merge.yml)
[![dependency status](https://deps.rs/repo/github/louib/nix2sbom/status.svg)](https://deps.rs/repo/github/louib/nix2sbom)
[![License file](https://img.shields.io/github/license/louib/nix2sbom)](https://github.com/louib/nix2sbom/blob/main/LICENSE)

`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation

> **Warning**
> This repo is still a work-in-progress.
  The command-line options and command names might change
  significantly until the project reaches version 1.0.0.

## Features
* Supports CycloneDX 1.4 format
* Supports JSON and YAML serialization formats
* Can generate a SBOM for your current `NixOS` system
* Detects and handles patches
* Discovers git URLs (using archive URLs)

## Using
### With flakes
The `-f` argument can be used to target a specific flake reference. For example, run the
following command to generate a SBOM for the default package of the flake in the current directory:
```
nix2sbom -f .#
```

### On NixOS
The SBOM for the current NixOS system can be obtained with the following command:
```
nix2sbom --current-system
```

### Logging
The `NIX2SBOM_LOG_LEVEL` environment variable can be used to tune the logging level.
The accepted values are `DEBUG`, `INFO`, `WARN` and `ERROR`. The default log level is `INFO`.

## Installing

### With Nix
Assuming that you have enabled both the `flakes` and `nix-command` experimental features:
```
nix profile install github:louib/nix2sbom
```

### With Cargo
```
cargo install --path .
```
