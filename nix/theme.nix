{
  lib,
  stdenv,
  finix-bootanim,
  image,
  width ? 1280,
  outroOn ? "All filesystems mounted",
}:

let
  introEnd = 46;
  loopEnd = 118;
  frames = 238;
in
stdenv.mkDerivation {
  pname = "finix-bootanim-theme";
  version = finix-bootanim.version;

  dontUnpack = true;
  nativeBuildInputs = [ finix-bootanim ];

  buildPhase = ''
    runHook preBuild
    finix_bootanim ${image} -o frames -w ${toString width}
    [ "$(ls frames | wc -l)" -eq ${toString frames} ] \
      || { echo "frame count != ${toString frames}: update constants in nix/theme.nix"; exit 1; }
    substitute ${./theme.script.in} finix-bootanim.script \
      --subst-var-by frameWidth ${toString width} \
      --subst-var-by introEnd ${toString introEnd} \
      --subst-var-by loopEnd ${toString loopEnd} \
      --subst-var-by frames ${toString frames} \
      --subst-var-by outroOn ${lib.escapeShellArg (builtins.toJSON outroOn)}
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    dir=$out/share/plymouth/themes/finix-bootanim
    mkdir -p $dir/images
    cp frames/*.png $dir/images/
    cp finix-bootanim.script $dir/
    cat > $dir/finix-bootanim.plymouth <<PLY
    [Plymouth Theme]
    Name=Finix Boot Animation
    Description=Animated Finix logo with a reveal of the final screen
    ModuleName=script

    [script]
    ImageDir=$dir/images
    ScriptFile=$dir/finix-bootanim.script
    PLY
    runHook postInstall
  '';

  meta = {
    description = "Finix boot animation plymouth theme";
    platforms = lib.platforms.linux;
  };
}
