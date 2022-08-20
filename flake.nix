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
        buildInputs = [
          pkgs.openssl
          pkgs.pkgconfig
          pkgs.rust-bin.stable.latest.default
          pkgs.clang
          pkgs.libclang.lib
          pkgs.godot
        ];
        nativeBuildInputs = [];
      in
      with pkgs;
      {
        devShell = mkShell {
          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          inherit buildInputs nativeBuildInputs;
          packages = [ rust-analyzer ];
        };
        defaultPackage = rustPlatform.buildRustPackage {
          name = "bevy-game-jam";
          version = "1.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "bevy_godot-0.2.0" = "sha256-TQl6VleUWgLDSnoqO+PyHB4pUqLGvEP2yw18buelp4A=";
            };
          };

          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
