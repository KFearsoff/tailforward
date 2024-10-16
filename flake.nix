{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dream2nix = {
      url = "github:nix-community/dream2nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs @ {...}:
  let
    pkgs = inputs.nixpkgs.legacyPackages.x86_64-linux;
    in {
      checks.x86_64-linux = {
        pre-commit-check = inputs.pre-commit-hooks.lib.x86_64-linux.run {
          src = ./.;
          hooks = {
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
      };

      devShells.x86_64-linux.default = pkgs.mkShellNoCC {
        packages = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
          cargo-deny
          cargo-edit
          cargo-nextest
        ];

        inherit (inputs.self.checks.x86_64-linux.pre-commit-check) shellHook;
      };

      packages.x86_64-linux.default = inputs.dream2nix.lib.evalModules {
        packageSets.nixpkgs = inputs.dream2nix.inputs.nixpkgs.legacyPackages.x86_64-linux;
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
}
