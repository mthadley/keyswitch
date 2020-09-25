{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "keyswitch";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "1j86p6zv7r809v05brid3j30zaxl8jvf76q70xz66g47ci4pckw8";

  verifyCargoDeps = true;

  nativeBuildInputs = with pkgs; [
    # Stuff for evdev-rs
    python3
    libtool
    pkgconfig
    autoconf
    automake
    libevdev
  ];

  meta = with pkgs.stdenv.lib; {
    description = "Remap keys";
    platforms = platforms.all;
  };
}
