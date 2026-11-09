# finix_bootanim

Renders the Finix boot animation to PNG frames (60 fps, 1920x1080 by default)

```
finix_bootanim <IMAGE> [-o frames] [-w 1920] [-p all|intro|loop|outro] [--frame N ...]
```

`IMAGE` is revealed at the end

If u want to make a make a video: `ffmpeg -framerate 60 -i frames/frame_%04d.png -pix_fmt yuv420p out.mp4`

## Finix plymouth theme

`flake.nix` ships a finix module that renders the frames at build time and installs them as a plymouth theme.

```nix
inputs.bootanim.url = "github:FixeQD/finix_bootanim";
inputs.bootanim.inputs.nixpkgs.follows = "nixpkgs";

modules = [ inputs.bootanim.finixModules.default ];

# config
programs.plymouth.bootanim = {
  enable = true;
  image = ./wallpaper.png; # revealed at the end
  # width = 1280;          # render width (16:9), bigger = bigger initrd
  # outroOn = "All filesystems mounted"; # plymouth message that starts the drop + reveal
};
```

This sets `programs.plymouth.enable`, `programs.plymouth.theme` and `settings.Daemon.Theme` for you.
The kernel needs `splash` (finix adds it when plymouth is enabled).
The timeline constants in `nix/theme.nix` must match `src/timeline.rs`; the build fails if the frame count drifts.
