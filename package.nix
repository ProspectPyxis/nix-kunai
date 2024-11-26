{
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "nix-kunai";
  inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) version; 

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
}
