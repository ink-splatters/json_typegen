{
  lib,
  inputs,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    options.craneLib = lib.mkOption {
      type = lib.types.attrs;
      default = let
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain config.rust-toolchain;
      in
        craneLib.overrideScope (_: _: {
          mkShell = pkgs.mkShell.override {inherit (config) stdenv;};
          stdenvSelector = _: config.stdenv;
        });
    };
  };
}
