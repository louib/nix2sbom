rec {
  description = "`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    naersk,
  }: (
    flake-utils.lib.eachDefaultSystem
    (
      system: (
        let
          authorName = "louib";
          mainBranch = "main";
          authorEmail = "code@louib.net";
          projectName = "nix2sbom";
          targetMuslSystem = "x86_64-unknown-linux-musl";

          pkgs = import nixpkgs {
            inherit system;
          };

          cargoPackages = with pkgs; [
            cargo
            cargo-outdated
            rustc
            rustfmt
            rust-analyzer
          ];

          # Defining our fenix-based Rust toolchain.
          fenixPkgs = fenix.packages.${system};
          toolchain = fenixPkgs.combine [
            fenixPkgs.minimal.cargo
            fenixPkgs.minimal.rustc
            fenixPkgs.targets.${targetMuslSystem}.latest.rust-std
          ];

          crossPkgs = naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          };
        in {
          devShells = {
            default = pkgs.mkShell {
              buildInputs = cargoPackages;
              shellHook = ''
              '';
            };
            # The musl shell is used to produce the statically-compiled binary.
            # It has only been tested on x86_64-linux systems.
            musl = pkgs.mkShell {
              buildInputs = [toolchain];
              shellHook = ''
                export CARGO_BUILD_TARGET="${targetMuslSystem}"
              '';
            };
          };
          packages = {
            default = pkgs.rustPlatform.buildRustPackage {
              pname = projectName;
              version = mainBranch;

              src = ./.;

              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              meta = with pkgs.lib; {
                inherit description;
                homepage = "https://github.com/${authorName}/${projectName}";
                license = licenses.gpl3;
                maintainers = [
                  {
                    name = authorName;
                    github = authorName;
                    email = authorEmail;
                  }
                ];
              };
            };
          };
        }
      )
    )
  );
}
