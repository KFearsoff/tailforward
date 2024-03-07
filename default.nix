{dream2nix, ...}: {
  imports = [
    # dream2nix modules go here
    dream2nix.modules.dream2nix.rust-cargo-lock
    dream2nix.modules.dream2nix.rust-crane
  ];

  mkDerivation = {
    src = ./.;
  };

  deps = {nixpkgs, ...}: {
    # dependencies go here
    inherit (nixpkgs) stdenv;
  };

  name = "tailforward";
  version = "0.7.0";

  # Ecosystem-dependent package definition goes here
}
