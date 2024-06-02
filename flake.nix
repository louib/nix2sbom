rec {
  description = "`nix2sbom` extracts the SBOM (Software Bill of Materials) from a Nix derivation";

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
          authorName = "louib";
          mainBranch = "main";
          authorEmail = "code@louib.net";
          projectName = "nix2sbom";

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
              # nativeBuildInputs = cargoPackages ++ [pkgs.glibc.static pkgs.pkg-config];
              buildInputs = cargoPackages;

              shellHook = ''
                # Even with that flag, the target has to be passed explicitly when building:
                # cargo build --release --target x86_64-unknown-linux-gnu
                # export RUSTFLAGS='-C target-feature=+crt-static'
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
