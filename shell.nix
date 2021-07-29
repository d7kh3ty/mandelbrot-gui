let
  rust-overlay = import (builtins.fetchTarball https://github.com/oxalica/rust-overlay/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ rust-overlay ]; config.allowUnfree = true;};
  rustNightly = nixpkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
    ];
  });
in
with nixpkgs;
nixpkgs.mkShell {
  #name = "rust-overlay-shell";
  buildInputs = [
    #lld
    rustNightly
    #clang
    #glibc
    cargo
    rustup
      #rustStableChannel
      # to use the latest nightly:
      #latest.rustChannels.nightly.rust
      #rust-bin.stable.latest.default
      # to use a specific nighly:
      #(nixpkgs.rustChannelOf { date = "2018-04-11"; channel = "nightly"; }).rust
      # to use the project's rust-toolchain file:
      #(nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; }).rust
    ];
  }
