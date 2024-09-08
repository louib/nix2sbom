# nix2sbom
![GitHub Release](https://img.shields.io/github/v/release/louib/nix2sbom)
[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/louib/nix2sbom/badge)](https://flakehub.com/flake/louib/nix2sbom)
[![GitHub](https://img.shields.io/badge/github-louib/nix2sbom-bb7a3652750d7dfd9ba196181cf30f809b3d7012?logo=github")](https://github.com/louib/nix2sbom)
[![Build Status](https://github.com/louib/nix2sbom/actions/workflows/merge.yml/badge.svg?branch=main)](https://github.com/louib/nix2sbom/actions/workflows/merge.yml)
[![Dependency Status](https://deps.rs/repo/github/louib/nix2sbom/status.svg)](https://deps.rs/repo/github/louib/nix2sbom)
[![License File](https://img.shields.io/github/license/louib/nix2sbom)](https://github.com/louib/nix2sbom/blob/main/LICENSE)

`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation

📚 [Documentation is here](https://github.com/louib/nix2sbom/wiki/Use-nix2sbom)

## Features
* Supports CycloneDX 1.4 format
* Supports SPDX 2.3 format (Experimental)
* Supports JSON and YAML serialization formats
* Generates a SBOM for your current `NixOS` system
* Detects and handles patches
* Discovers git URLs (using archive URLs)

## Using
### In GitHub Actions
Here is an example of how to generate an SPDX manifest for your nix flake in a GHA workflow:
```
  generate-sbom-manifests:
    name: Generate SPDX SBOM manifest
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Nix
        uses: DeterminateSystems/nix-installer-action@ab6bcb2d5af0e904d04aea750e2089e9dc4cbfdd # v13

      - name: Install nix2sbom
        uses: EricCrosson/install-github-release-binary@681cc3de7c5c5ac935b1a2a19e4e0c577c4d3027 # v2.3.4
        with:
          targets: louib/nix2sbom/nix2sbom@v2.1.2

      - name: Generate the SPDX manifest
        run: |
          nix2sbom .# -f spdx > sbom.spdx
```

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
