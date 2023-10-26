{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dream2nix = {
      url = "github:nix-community/dream2nix/legacy";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-parts.follows = "flake-parts";
      };
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
        inputs',
        ...
      }: {
        devenv.shells.default = {
          languages.rust.enable = true;
          languages.rust.version = "latest";

          # https://github.com/cachix/devenv/issues/528
          containers = pkgs.lib.mkForce {};

          packages = with pkgs; [
            cargo-deny
            cargo-edit
            cargo-nextest
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

        dream2nix.inputs."tailforward" = let
          # we use the full toolchain derivation here as using
          # only the cargo / rustc derivation *does not* work.
          inherit (inputs'.fenix.packages.minimal) toolchain;
        in {
          source = ./.;
          projects = builtins.fromTOML (builtins.readFile ./projects.toml);
          packageOverrides = {
            # for crane builder
            "^.*".set-toolchain.overrideRustToolchain = _: {cargo = toolchain;};
          };
        };

        inherit (config.dream2nix.outputs."tailforward") packages;
      };
    };
}
