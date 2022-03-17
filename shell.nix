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
      llvmPackages.bintools
      pkg-config
      gettext
      openssl
      postgresql
      age
      apacheHttpd

      rust-analyzer
      cargo-edit
      cargo-feature
      cargo-udeps
      cargo-bloat
      linuxPackages.perf
      docker-compose
      # (rust-bin.nightly.latest.default.override {
      #   extensions = ["rust-src" "clippy"];
      # })
      (rust-bin.stable."1.59.0".default.override {
        extensions = ["rust-src" "clippy"];
      })
    ];

    RUST_BACKTRACE = 1;

    # optional lld setup for faster compilation
    RUSTFLAGS = "-Clink-arg=-fuse-ld=lld";
  }
