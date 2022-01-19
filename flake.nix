{
  description = "arne.me";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    gitignore.url = "github:hercules-ci/gitignore.nix";
  };

  outputs = { self, nixpkgs, flake-utils, gitignore }:
  flake-utils.lib.eachDefaultSystem (system:
    let 
      inherit (gitignore.lib) gitignoreSource;
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      defaultPackage = pkgs.stdenvNoCC.mkDerivation {
        name = "static";
        src = gitignoreSource self;
        buildInputs = [ pkgs.hugo pkgs.minify pkgs.fd ];
        buildPhase = ''
          hugo --minify -d static
          fd '.html$' static -x minify --html-keep-document-tags --html-keep-end-tags -o {} {}
        '';
        installPhase = ''
          cp -r static $out
        '';
      };

      devShell = pkgs.mkShell {
        buildInputs = [ pkgs.hugo ];
      };
    });
}
