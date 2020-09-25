{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  inputsFrom = [
    (import ./. { inherit pkgs; })
  ];

  nativeBuildInputs = with pkgs; [
    rustfmt
  ];
}
