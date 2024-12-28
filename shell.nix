let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> { overlays = [ (import rustOverlay) ]; };
in
pkgs.mkShell rec {
  buildInputs =
    with pkgs;
    [
      nixfmt-classic
      mold
      clang
      pkg-config
      gettext
      age
      apacheHttpd
      vegeta

      openssl
      postgresql
      (rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "clippy"
        ];
      })
      protobuf
      sqlx-cli

      rust-analyzer
      cargo-edit
      cargo-feature
      cargo-udeps
      cargo-bloat
      docker-compose
    ]
    ++ (pkgs.lib.optionals pkgs.stdenv.isLinux [ linuxPackages.perf ]);

  RUST_BACKTRACE = 1;
  MOLD_PATH = "${pkgs.mold.out}/bin/mold";
  RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
