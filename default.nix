{ pkgs ? import <nixpkgs> {}}:

pkgs.stdenvNoCC.mkDerivation {
  name = "arne.me";

  src = with builtins; filterSource (path: type: substring 0 1 
    (baseNameOf path) != "." && (baseNameOf path) != "default.nix" && type != "symlink")
    ./.;

  dontConfigure = true;
  buildInputs = [ pkgs.hugo pkgs.minify pkgs.fd ];
  preferLocalBuild = true;
  installPhase = ''
    runHook preInstall
    hugo --minify -d $out
    fd '.html$' $out -x minify --html-keep-document-tags --html-keep-end-tags -o {} {}
    runHook postInstall
  '';
}