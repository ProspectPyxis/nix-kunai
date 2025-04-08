{
  description = "Yet another nix dependency manager - simple code, but robust handling.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    systems.url = "github:nix-systems/default";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    systems,
    fenix,
    ...
  }: let
    forEachSystem = function: let
      overlays = [fenix.overlays.default];
    in
      nixpkgs.lib.genAttrs (import systems)
      (system:
        function (import nixpkgs {
          inherit system overlays;
        }));
  in {
    packages = forEachSystem (pkgs: rec {
      default = pkgs.callPackage ./package.nix {};
      nix-kunai = default;
    });

    devShells = forEachSystem (pkgs: {
      default = pkgs.mkShellNoCC {
        packages = with pkgs; [
          git
          nix
          (fenix.default.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rust-std"
            "rustc"
            "rustfmt"
          ])
        ];
      };
    });
  };
}
