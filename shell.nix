let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  # pinnedPkgs = (import <nixpkgs>).fetchFromGitHub {
  #   owner  = "NixOS";
  #   repo   = "nixpkgs";
  #   rev    = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
  #   sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  # };
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    mold
    clang
    pkg-config
    gettext
    age
    apacheHttpd
    linuxPackages.perf
    vegeta

    openssl
    postgresql
    (rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" "clippy" ];
    })
    protobuf
    sqlx-cli

    rust-analyzer
    cargo-edit
    cargo-feature
    cargo-udeps
    cargo-bloat
    docker-compose
  ];

  RUST_BACKTRACE = 1;
  MOLD_PATH = "${pkgs.mold.out}/bin/mold";
  RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
