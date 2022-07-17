{
  description = "arne.me";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
  flake-utils.lib.eachDefaultSystem (system:
    let
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
        src = self;
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
