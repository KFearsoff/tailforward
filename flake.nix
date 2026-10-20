{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      pre-commit-hooks,
      ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      inherit (nixpkgs) lib;
      package = import ./package.nix { inherit pkgs lib; };
    in
    {
      checks.x86_64-linux = {
        pre-commit-check = pre-commit-hooks.lib.x86_64-linux.run {
          src = ./.;
          hooks = {
            # Nix
            nixfmt.enable = true;
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

      packages.x86_64-linux = {
        default = package;
        tailforward = package;
      };
    };
}
