{
  description = "Finix boot animation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      rustPlatformFor =
        pkgs:
        let
          toolchain = (pkgs.extend rust-overlay.overlays.default).rust-bin.stable.latest.minimal;
        in
        pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };
    in
    {
      packages = forAllSystems (pkgs: rec {
        finix-bootanim = pkgs.callPackage ./nix/package.nix { rustPlatform = rustPlatformFor pkgs; };
        default = finix-bootanim;
      });

      overlays.default = final: _prev: {
        finix-bootanim = final.callPackage ./nix/package.nix { rustPlatform = rustPlatformFor final; };
      };

      finixModules.default = import ./nix/module.nix self;

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
          packages = [ pkgs.rust-analyzer pkgs.clippy pkgs.rustfmt ];
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);
    };
}
