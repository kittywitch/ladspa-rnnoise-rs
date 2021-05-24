{ pkgs ? import <nixpkgs> { } }: (import <arc> { inherit pkgs; }).shells.rust.nightly.overrideAttrs ( old: {
  nativeBuildInputs = old.nativeBuildInputs or [] ++ [
    pkgs.pkg-config
  ];
  buildInputs =  old.buildInputs ++ [
    pkgs.openssl
  ];
})
