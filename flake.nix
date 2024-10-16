{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dream2nix = {
      url = "github:nix-community/dream2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      pre-commit-hooks,
      dream2nix,
      ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in
    {
      checks.x86_64-linux = {
        pre-commit-check = pre-commit-hooks.lib.x86_64-linux.run {
          src = ./.;
          hooks = {
            # Nix
            nixfmt-rfc-style.enable = true;
            deadnix.enable = true;
            statix.enable = true;

            # Rust
            cargo-check.enable = true;
            clippy = {
              enable = true;
              raw.verbose = true;
            };
            rustfmt.enable = true;
          };
        };
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          cargo-deny
          cargo-edit
          cargo-nextest
        ];

        inherit (inputs.self.checks.x86_64-linux.pre-commit-check) shellHook;
      };

      packages.x86_64-linux.default = dream2nix.lib.evalModules {
        packageSets.nixpkgs = dream2nix.inputs.nixpkgs.legacyPackages.x86_64-linux;
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
