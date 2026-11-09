self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.plymouth.bootanim;
  finix-bootanim = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
in
{
  options.programs.plymouth.bootanim = {
    enable = lib.mkEnableOption "the Finix boot animation plymouth theme";

    image = lib.mkOption {
      type = lib.types.path;
      example = lib.literalExpression "./wallpaper.png";
      description = "Final screen revealed at the end of the animation (png, jpeg or webp), usually your wallpaper.";
    };

    width = lib.mkOption {
      type = lib.types.ints.positive;
      default = 1280;
      description = ''
        Width in pixels the frames are rendered at (16:9). Plymouth scales them to the screen.
        Higher values make the initrd bigger.
      '';
    };

    outroOn = lib.mkOption {
      type = lib.types.str;
      default = "All filesystems mounted";
      description = ''
        Plymouth message that starts the final part of the animation (the drop and the reveal).
      '';
    };

    package = lib.mkOption {
      type = lib.types.package;
      readOnly = true;
      default = pkgs.callPackage ./theme.nix {
        inherit finix-bootanim;
        inherit (cfg) image width outroOn;
      };
      description = "The generated plymouth theme.";
    };
  };

  config = lib.mkIf cfg.enable {
    programs.plymouth = {
      enable = true;
      theme = cfg.package;
      settings.Daemon.Theme = "finix-bootanim";
    };
  };
}
