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
      url = "github:nix-community/dream2nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: {
        devenv.shells.default = {
          languages.rust.enable = true;
          languages.rust.channel = "nixpkgs";

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

        packages.default = inputs.dream2nix.lib.evalModules {
          packageSets.nixpkgs = inputs.dream2nix.inputs.nixpkgs.legacyPackages.${system};
          modules = [
            ./default.nix
            {
              paths = {
                projectRoot = ./.;
                projectRootFile = "flake.nix";
                package = ./.;
              };
            }
          ];
        };
      };
    };
}
