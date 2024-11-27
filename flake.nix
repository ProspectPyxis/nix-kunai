{
  description = "Yet another nix dependency manager - simple code, but robust handling.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    systems.url = "github:nix-systems/default";
  };

  outputs = {
    nixpkgs,
    systems,
    ...
  }: let
    forEachSystem = function:
      nixpkgs.lib.genAttrs (import systems)
      (system: function nixpkgs.legacyPackages.${system});
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
        ];
      };
    });
  };
}
