{ pkgs ? import <nixpkgs> {}}:

pkgs.stdenvNoCC.mkDerivation {
  name = "arne.me";

  src = with builtins; filterSource (path: type: substring 0 1 
    (baseNameOf path) != "." && (baseNameOf path) != "default.nix" && type != "symlink")
    ./.;

  dontConfigure = true;
  buildInputs = [ pkgs.hugo ];
  preferLocalBuild = true;
  installPhase = ''
    runHook preInstall
    hugo -d $out
    runHook postInstall
  '';
}