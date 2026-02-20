{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  pkgs ? import nixpkgs { inherit system; },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    nodejs_22
    nixd
    nixfmt-rfc-style
  ];

  shellHook = ''
    export PATH="$PWD/node_modules/.bin:$PATH"
  '';
}
