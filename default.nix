{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "keyswitch";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "1abjffz771svi6dvywr1cfjh8aw8pqrsy5k39jsffrgl520zwkwl";

  verifyCargoDeps = true;

  meta = with pkgs.stdenv.lib; {
    description = "Remap keys";
    platforms = platforms.all;
  };
}
