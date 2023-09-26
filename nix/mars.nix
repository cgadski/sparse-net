{
  lib,
  stdenv,
  fetchFromGitHub,
  graphviz,
  lapack,
  libGL,
  libGLU,
  freeglut,

  darwin,
}:
stdenv.mkDerivation {
  name = "mars";

  src = fetchFromGitHub {
    owner = "marckhoury";
    repo = "mars";
    rev = "3e12b09a3f0b0f63e46db2cf1ef7547d90ff1380";
    hash = "sha256-zcJ6XfYQUNEYLcOMKxXUJLHwTSF8fNwm3lgeM4cK9i4=";
  };

  patches = [./mars.patch];

  buildPhase = ''
  source build.sh
  '';

  installPhase = ''
  mkdir -p $out/bin/
  cp ./mars $out/bin/mars
  '';

  buildInputs = [
    graphviz
    lapack
    libGL
  ] ++ (lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.GLUT
    darwin.apple_sdk.frameworks.Cocoa
  ]) ++ (lib.optionals stdenv.isLinux [
    libGLU
    freeglut
  ]);

  meta = with lib; {
    description = "A graph drawing tool for large graph visualization";
    homepage = "https://github.com/marckhoury/mars";
    license = licenses.epl10;
  };
}
