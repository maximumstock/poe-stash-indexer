{
  description = "poe-stash-indexer";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , rust-overlay
    , crane
    ,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        craneLib = crane.mkLib pkgs;
      in
      {
        devShell = import ./ci.nix { inherit pkgs; };
        formatter = pkgs.nixpkgs-fmt;
        packages.indexer = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          pname = "indexer";
          cargoExtraArgs = "-p indexer";
          cargoClippyExtraArgs = "--all-features --all-targets -- -D warnings";
        };
      }
    );
}
