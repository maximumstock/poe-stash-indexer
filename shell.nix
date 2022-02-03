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
  pkgs.mkShell {
    buildInputs = with pkgs; [
      pkg-config
      gettext
      openssl
      openssl.dev
      rust-analyzer
      cargo-edit
      cargo-feature
      linuxPackages.perf
      docker-compose
      linuxPackages.perf
      cargo-edit
      rust-analyzer
      age
      apacheHttpd
      postgresql
      (rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "clippy"];
      })
      lld
    ];

    RUST_BACKTRACE = 1;
    RUSTFLAGS = "-Clink-arg=-fuse-ld=lld" ;
  }
