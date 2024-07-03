{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.pkg-config
    pkgs.libxkbcommon
    pkgs.rustc
    pkgs.cargo
    pkgs.udev
  ];

  RUSTFLAGS = "-L${pkgs.libxkbcommon}/lib -L${pkgs.udev}/lib";
}

