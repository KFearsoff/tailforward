{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    dream2nix.url = "github:nix-community/dream2nix";
    dream2nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    devenv,
    systems,
    dream2nix,
    ...
  } @ inputs: let
    forEachSystem = nixpkgs.lib.genAttrs (import systems);
  in
    {
      devShells =
        forEachSystem
        (system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {
                languages.rust.enable = true;

                pre-commit.hooks = {
                  # Nix
                  alejandra.enable = true;
                  deadnix.enable = true;
                  statix.enable = true;

                  # Rust
                  cargo-check.enable = true;
                  clippy.enable = true;
                  clippy.raw.args = ["-W" "clippy::pedantic" "-W" "clippy::nursery" "-W" "clippy::unwrap_used" "-W" "clippy::expect_used"];
                  rustfmt.enable = true;
                };
              }
            ];
          };
        });
    }
    // (dream2nix.lib.makeFlakeOutputs {
      systems = ["x86_64-linux"];
      config.projectRoot = ./.;
      source = ./.;
      projects = ./projects.toml;
    });
}
