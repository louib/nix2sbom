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
* Generation of a SBOM based on the CycloneDX 1.4 format
* Generation of a SBOM for your current `NixOS` system
* Patch detection
* Detection of the git URLs based on the archive URLs

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

## Using

### Logging
The `NIX2SBOM_LOG_LEVEL` environment variable can be used to tune the logging level.
The accepted values are `DEBUG`, `INFO`, `WARN` and `ERROR`. The default log level is `INFO`.
