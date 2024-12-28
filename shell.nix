let
  rustOverlay = builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> { overlays = [ (import rustOverlay) ]; };
in pkgs.mkShell rec {
  buildInputs = with pkgs;
    [
      nixfmt-classic
      pkg-config
      gettext
      age
      vegeta
      protobuf
      docker-compose

      (rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "clippy" ];
      })

      rust-analyzer
      cargo-edit
      cargo-feature
      cargo-udeps
      cargo-bloat
    ] ++ (pkgs.lib.optionals pkgs.stdenv.isLinux [
      clang
      linuxPackages.perf
      mold
    ]);

  RUST_BACKTRACE = 1;

} // pkgs.lib.mkIf (pkgs.stdenv.isLinux) rec {
  MOLD_PATH = "${pkgs.mold.out}/bin/mold";
  RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
