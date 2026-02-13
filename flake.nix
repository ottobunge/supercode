{
  description = "Supercode - Multi-agent orchestration system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          overlays = [ rust-overlay ];
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            (pkgs.rust-bin.fromRustbinToolchainFile ./rust-toolchain.toml)
            cargo
            rustfmt
            clippy

            # SQLite
            sqlite

            # For building
            pkg-config

            # Optional: for running tests
            cacert
          ];

          # Environment variables
          DATABASE_PATH = "${HOME}/.supercode/supercode.db";

          # Shell hook
          shellHook = ''
            echo "Supercode dev shell"
            echo "Database path: $DATABASE_PATH"
            mkdir -p ~/.supercode
          '';
        };

        packages.supercode = pkgs.rustPlatform.buildRustPackage {
          pname = "supercode";
          version = "0.1.0";
          src = ./.;
          cargoLock = null;
          buildType = "release";
          cargoHash = "sha256-00000000000000000000000000000000000000000000000000";

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.sqlite ];

          meta = with pkgs.lib; {
            description = "Multi-agent orchestration system for managing coding agent sessions";
            homepage = "https://github.com/username/supercode";
            license = licenses.mit;
            mainProgram = "supercode";
          };
        };
      }
    );
}
