{
  description = "co: a coding agent harness (Rust server, tool hosts and TUI; SolidJS web; Tauri desktop)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        # The toolchain pinned in rust-toolchain.toml (shared with rustup users).
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # What Tauri needs to build and run the desktop app on Linux.
        tauriLinuxDeps = with pkgs; [
          webkitgtk_4_1
          gtk3
          libsoup_3
          glib
          cairo
          pango
          gdk-pixbuf
          atk
          librsvg
          openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            [
              rust
              pkgs.pkg-config
              pkgs.cargo-tauri
              pkgs.sqlx-cli
              pkgs.just
              pkgs.nodejs_26
              pkgs.pnpm
              pkgs.postgresql_18 # psql, pg_dump
              pkgs.patchelf # scripts/nixos-patch-node-bins.sh
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux tauriLinuxDeps;

          DATABASE_URL = "postgres://co:co@localhost:5432/co";
          CO_SERVER_URL = "ws://localhost:7420/ws";
          RUST_LOG = "info,co=debug";
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
