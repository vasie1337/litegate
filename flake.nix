{
  description = "LTC Payment Processor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        nativeBuildInputs = with pkgs; [
          rustVersion
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          sqlite
        ];

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = cargoToml.package.name or "ltc-payments";
          version = cargoToml.package.version or "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs buildInputs;
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          
          shellHook = ''
            echo "🦀 Rust development environment"
            export RUST_BACKTRACE=1
            export RUST_LOG=info
            
            # Required for the HMAC crypto
            export LD_LIBRARY_PATH=${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH
            
            # For testing, set these dummy values
            export MAIN_ADDRESS="not-set"
            export AES_KEY="0000000000000000000000000000000000000000000000000000000000000000"
            export WEBHOOK_URL=""
            export WEBHOOK_SECRET=""
          '';
        };
      }
    );
}