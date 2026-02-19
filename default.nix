{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  fenix ? import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { },
  target ? "x86_64-unknown-linux-musl",
}:

let
  pkgs = import nixpkgs { inherit system; };
  crossPkgs = import nixpkgs {
    inherit system;
    crossSystem = {
      config = target;
      isStatic = true;
    };
  };

  rustToolchain = with fenix; combine [
    latest.rustc
    latest.cargo
    targets.${target}.latest.rust-std
  ];

  rustPlatform = crossPkgs.makeRustPlatform {
    rustc = rustToolchain;
    cargo = rustToolchain;
  };

in
rustPlatform.buildRustPackage {
  pname = "bilbo";
  version = "0.1.0";

  src = pkgs.nix-gitignore.gitignoreSource [ ] ./.;

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ crossPkgs.openssl ];

  buildNoDefaultFeatures = true;
  buildFeatures = [ "ssr" ];
  cargoBuildFlags = [ "--bin" "bilbo" ];

  doCheck = false;
}
