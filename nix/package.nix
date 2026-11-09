{ lib, rustPlatform }:

let
  manifest = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  inherit (manifest.package) version;

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    inherit (manifest.package) description;
    license = lib.licenses.mit;
    mainProgram = manifest.package.name;
  };
}
