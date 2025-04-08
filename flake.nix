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
        function system (import nixpkgs {
          inherit system overlays;
        }));
  in {
    packages = forEachSystem (system: pkgs: rec {
      default = pkgs.callPackage ./package.nix {
        toolchain = fenix.packages.${system}.minimal.toolchain;
      };
      nix-kunai = default;
    });

    devShells = forEachSystem (_system: pkgs: {
      default = import ./shell.nix {inherit pkgs;};
    });
  };
}
