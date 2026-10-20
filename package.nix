{
  lib,
  pkgs,
  doCheck ? true,
}:
let
  props = fromTOML (builtins.readFile ./Cargo.toml);
  inherit (props.package) version;
in
pkgs.rustPlatform.buildRustPackage {
  pname = "tailforward";
  inherit version doCheck;
  src = lib.cleanSource ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
  meta = with lib; {
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "tailforward";
  };
}
