{rustPlatform}:
rustPlatform.buildRustPackage {
  pname = "nix-kunai";
  version = "0.1.0";

  src = ./.;

  cargoLock = ./Cargo.lock;
}
