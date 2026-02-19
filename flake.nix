{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    fenix.url = "github:nix-community/fenix/monthly";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      {
        packages.default = import ./default.nix {
          inherit nixpkgs system;
          fenix = fenix.packages.${system};
        };

        devShells.default = import ./shell.nix {
          inherit nixpkgs system;
          rust-overlay = import rust-overlay;
        };
      });
}
