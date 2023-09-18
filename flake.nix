let
  authorName = "louib";
  authorEmail = "code@louib.net";
  projectName = "nix2sbom";
  description = "`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation";
in {
  inherit description;

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }: (
    flake-utils.lib.eachDefaultSystem (
      system: (
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          cargoPackages = with pkgs; [
            cargo
            rustc
            rustfmt
          ];
        in {
          devShells = {
            default = pkgs.mkShell {
              buildInputs = cargoPackages;

              shellHook = ''
              '';
            };
          };
          packages = {
            default = pkgs.rustPlatform.buildRustPackage rec {
              pname = projectName;
              version = "main";

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
