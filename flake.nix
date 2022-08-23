{
  description = "Bevy Game Jam #2 - Combine <https://itch.io/jam/bevy-jam-2>";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };

        buildInputs = [
          pkgs.openssl
          pkgs.pkgconfig
          pkgs.rust-bin.stable.latest.default
          pkgs.clang
          pkgs.libclang.lib
          pkgs.godot
        ];
        nativeBuildInputs = [
          pkgs.rustPlatform.bindgenHook
        ];
      in
      {
        devShell = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          packages = [ rust pkgs.rust-analyzer ];
        };
        defaultPackage = rustPlatform.buildRustPackage {
          name = "bevy-game-jam";
          version = "1.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "bevy_godot-0.2.2" = "sha256-OAgAhhmR4Atz05EuFNliJ5sDzJ5CZl0xhaEMU6dZAFA=";
            };
          };

          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
