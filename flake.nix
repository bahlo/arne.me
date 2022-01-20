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
      buildInputs = [ pkgs.zola ];
    in {
      defaultPackage = pkgs.stdenvNoCC.mkDerivation {
        name = "static";
        src = gitignoreSource self;
        inherit buildInputs;
        buildPhase = ''
          zola build
        '';
        installPhase = ''
          cp -r public $out
        '';
      };

      devShell = pkgs.mkShell {
        inherit buildInputs;
      };
    });
}
