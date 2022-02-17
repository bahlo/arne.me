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
      buildInputs = [ 
        pkgs.zola
        pkgs.python3
        pkgs.python3Packages.colormath
        pkgs.python3Packages.toml
      ];
      shortRev = if (self ? shortRev) then self.shortRev else "dirty";
    in {
      defaultPackage = pkgs.stdenvNoCC.mkDerivation {
        pname = "arne.me";
        version = "0.${shortRev}";
        src = gitignoreSource self;
        inherit buildInputs;
        buildPhase = ''
          python3 ./scripts/embed_revision.py ${shortRev}
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
