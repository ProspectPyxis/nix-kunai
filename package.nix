{
  pkgs,
  lib,
  rustPlatform,
  makeWrapper,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

  pname = cargoToml.package.name;
in
  rustPlatform.buildRustPackage {
    inherit pname;
    inherit (cargoToml.package) version;

    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    buildInputs = [
      makeWrapper
    ];

    postFixup = ''
      wrapProgram $out/bin/${pname} \
        --set PATH ${lib.makeBinPath (with pkgs; [
        nix
        git
      ])}
    '';
  }
