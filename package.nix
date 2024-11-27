{
  pkgs,
  lib,
  rustPlatform,
  openssl,
  pkg-config,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

  pname = cargoToml.package.name;
in
rustPlatform.buildRustPackage {
  inherit pname;
  inherit (cargoToml.package) version; 

  src = ./.;

  buildInputs = [
    openssl
  ];
  nativeBuildInputs = [
    pkg-config
  ];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  postFixup = ''
    wrapProgram $out/bin/${pname} \
      --set PATH ${lib.makeBinPath (with pkgs; [
        nix
        git
      ])}
  '';
}
