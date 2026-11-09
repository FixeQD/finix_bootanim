# finix_bootanim

Renders the Finix boot animation to PNG frames (60 fps, 1920x1080 by default)

```
finix-frames <IMAGE> [-o frames] [-w 1920] [-p all|intro|loop|outro] [--frame N ...]
```

`IMAGE` is revealed at the end

If u want to make a make a video: `ffmpeg -framerate 60 -i frames/frame_%04d.png -pix_fmt yuv420p out.mp4`
