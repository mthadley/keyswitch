{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "keyswitch";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "0am5dw8pkspnrd5nvhbs7lz3aazr33cgf0dax5ybd7hr9gbf0bs3";

  verifyCargoDeps = true;

  meta = with pkgs.stdenv.lib; {
    description = "Remap keys";
    platforms = platforms.all;
  };
}
