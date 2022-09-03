{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell { buildInputs = with pkgs; [ stdenv pkg-config openssl.dev ]; }
