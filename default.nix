{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "keyswitch";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "0zi6whcs3ik18d0nsz0s635i7xgprwvhd4dinl4m3x471sg1b7w4";

  verifyCargoDeps = true;

  meta = {
    description = "Remap keys";
    platforms = pkgs.lib.platforms.linux;
  };
}
