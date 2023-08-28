# nix2sbom
[![Build Status](https://github.com/louib/nix2sbom/actions/workflows/merge.yml/badge.svg?branch=main)](https://github.com/louib/nix2sbom/actions/workflows/merge.yml)
[![dependency status](https://deps.rs/repo/github/louib/nix2sbom/status.svg)](https://deps.rs/repo/github/louib/nix2sbom)
[![License file](https://img.shields.io/github/license/louib/nix2sbom)](https://github.com/louib/nix2sbom/blob/main/LICENSE)

`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation

📚 Documentation for using `nix2sbom` [is here](https://github.com/louib/nix2sbom/wiki/Use-nix2sbom)

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
