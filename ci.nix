{ pkgs }:
pkgs.mkShell
  rec {

    buildInputs =
      with pkgs;
      [
        clang
        nixfmt-classic
        pkg-config
        gettext
        age
        vegeta
        docker-compose

        (rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "clippy"
          ];
        })

        cargo-edit
        cargo-feature
        cargo-udeps
        cargo-bloat
      ]
      ++ (pkgs.lib.optionals pkgs.stdenv.isLinux [
        linuxPackages.perf
        mold
      ])
      ++ (pkgs.lib.optionals pkgs.stdenv.isDarwin [
        # Additional darwin specific inputs can be set here
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      ]);

    RUST_BACKTRACE = 1;

  }
  // pkgs.lib.mkIf (pkgs.stdenv.isLinux) rec {
  MOLD_PATH = "${pkgs.mold.out}/bin/mold";
  RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
