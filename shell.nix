{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  rust-overlay ? import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"),
}:

let
  pkgs = import nixpkgs {
    inherit system;
    overlays = [ rust-overlay ];
  };
  rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
    targets = [ "wasm32-unknown-unknown" ];
  };

in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustToolchain
    cargo-leptos
    sqlx-cli
    pkg-config
    openssl
    dart-sass
    binaryen
    wasm-bindgen-cli
  ];

  shellHook = ''
    echo "bilbo dev shell ready"
  '';
}
