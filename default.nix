{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "keyswitch";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "12b5if534ihkgc00m1nfqgaph99p719nj2c2fdjj12svsgd9x1d5";

  verifyCargoDeps = true;

  meta = with pkgs.stdenv.lib; {
    description = "Remap keys";
    platforms = platforms.all;
  };
}
