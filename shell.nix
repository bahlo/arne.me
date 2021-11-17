with import <nixpkgs> {};

mkShell {
  nativeBuildInputs = with buildPackages; [
    hugo
  ];
}
