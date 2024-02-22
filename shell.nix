# A developement shell for compiling Rust.
# This file needs to be in the root of the project that uses Rust, and requires a rust-toolchain.
#
# For installation steps see: https://nix.dev/tutorials/install-nix
#
# To start the dev shell, simply run: `nix-shell`
#
# By default `nix-shell` opens a new `bash` shell. If you would like to use
# your own, for example `zsh`, you can open a new instance directly after: `zsh`.
# For more information on this limitation see https://nixos.wiki/wiki/Development_environment_with_nix-shell#direnv
{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {} }:

let
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {};
in
pkgs.mkShell {
  name = "rust-dev";

  nativeBuildInputs = [
    # rust toolchain
    (fenix.fromToolchainFile { dir = ./.; })
    pkgs.pkg-config
    pkgs.clang
  ];

  buildInputs = with pkgs; [
    # dev tools
    which
    htop
    zlib

    # build dependencies
    rocksdb
    openssl.dev
    libiconv
  ] ++ lib.optionals stdenv.isDarwin [darwin.apple_sdk.frameworks.Security darwin.apple_sdk.frameworks.SystemConfiguration];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
    export ROCKSDB_LIB_DIR="${pkgs.rocksdb}/lib";
  '';
}