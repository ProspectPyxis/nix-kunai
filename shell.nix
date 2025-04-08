{ pkgs }:

pkgs.mkShellNoCC {
  packages = with pkgs; [
    git
    nix
    (fenix.default.withComponents [
      "cargo"
      "clippy"
      "rust-std"
      "rustc"
      "rustfmt"
    ])
  ];
}
