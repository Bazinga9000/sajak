{
  lib,
  rustPlatform,
  fetchFromGitHub,
  fetchurl,
  pkg-config,
  udev,
  vulkan-loader,
  libX11,
  libXcursor,
  libxcb,
  libXi,
  libxkbcommon,
  stdenv,
  darwin,
  alsa-lib,
  makeBinaryWrapper,
}:
let
  corpusVersion = "2024-10-20";
in
rustPlatform.buildRustPackage rec {
  pname = "sajak";
  version = "0.1.0";

  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    makeBinaryWrapper
  ];

  corpus = fetchurl {
    url = "https://github.com/Bazinga9000/sajak/releases/download/corpus-${corpusVersion}/trie.sjt";
    hash = "sha256-HOz2rk5C5Zfe84byMosm4sD3Tiont0eluymnh3VOZ28=";
  };

  postInstall = ''
    rm $out/bin/timing
    wrapProgram $out/bin/nu_plugin_sajak --set SAJAK_DEFAULT_TRIE $corpus
    wrapProgram $out/bin/sajak_http --set SAJAK_DEFAULT_TRIE $corpus
    wrapProgram $out/bin/sajak --set SAJAK_DEFAULT_TRIE $corpus
  '';

  meta = {
    description = "Puzzlehunt tool to search corpora for regular expressions.";
    homepage = "https://github.com/Bazinga9000/sajak";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ ];
    mainProgram = "sajak";
  };
}
