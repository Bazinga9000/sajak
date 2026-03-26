{
  description = "Puzzlehunt tool for finding words and phrases.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      top@{
        config,
        withSystem,
        moduleWithSystem,
        ...
      }:
      {

        systems = [
          "x86_64-linux"
          # If anyone's out there using this flake on any other system type, feel free to PR in other systems you might need.
          # I can't verify that these builds work on such systems, which is why they aren't already here.
        ];

        imports = [
          inputs.flake-parts.flakeModules.easyOverlay
        ];

        perSystem =
          { config, pkgs, ... }:
          {
            overlayAttrs = config.packages;

            packages = {
              sajak = pkgs.callPackage ./nix/package.nix { };
            };
          };
      }
    );
}
