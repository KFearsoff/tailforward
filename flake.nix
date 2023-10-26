{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    flake-parts.url = "github:hercules-ci/flake-parts";

    dream2nix = {
      url = "github:nix-community/dream2nix/legacy";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
        inputs.dream2nix.flakeModuleBeta
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      dream2nix = {
        config.projectRoot = ./.;
      };

      perSystem = {
        config,
        pkgs,
        ...
      }: {
        devenv.shells.default = {
          languages.rust.enable = true;

          # https://github.com/cachix/devenv/issues/528
          containers = pkgs.lib.mkForce {};

          packages = with pkgs; [
            cargo-deny
            cargo-edit
          ];

          pre-commit.hooks = {
            # Nix
            alejandra.enable = true;
            deadnix.enable = true;
            statix.enable = true;

            # Rust
            cargo-check.enable = true;
            clippy = {
              enable = true;
              raw.verbose = true;
              raw.args = ["-D" "clippy::pedantic" "-D" "clippy::nursery" "-D" "clippy::unwrap_used" "-D" "clippy::expect_used"];
            };
            rustfmt.enable = true;
          };
        };

        dream2nix.inputs."tailforward" = {
          source = ./.;
          projects = builtins.fromTOML (builtins.readFile ./projects.toml);
        };

        inherit (config.dream2nix.outputs."tailforward") packages;
      };
    };
}
