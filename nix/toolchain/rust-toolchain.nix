{lib, ...}: {
  perSystem = {
    config,
    inputs',
    ...
  }: let
    fenix = inputs'.fenix.packages;
    fenixDefault = fenix.default;
    rustTarget = config.stdenv.hostPlatform.rust.rustcTarget;
    wasmRustStd = fenix.targets.wasm32-unknown-unknown.latest.rust-std;

    darwinRustc = fenixDefault.rustc-unwrapped.overrideAttrs (old: {
      postFixup =
        (old.postFixup or "")
        + ''
          # Fenix Darwin rust-objcopy can miss its bundled libLLVM rpath.
          # https://github.com/nix-community/fenix/issues/242
          install_name_tool -add_rpath "$out/lib" "$out/lib/rustlib/${rustTarget}/bin/rust-objcopy"
        '';
    });

    darwinToolchain = fenixDefault.toolchain.overrideAttrs (_old: {
      paths = [
        fenixDefault.cargo
        fenixDefault.clippy-preview-unwrapped
        fenixDefault.rust-docs
        fenixDefault.rust-std
        darwinRustc
        fenixDefault.rustfmt-preview
      ];
    });
  in {
    options.rust-toolchain = lib.mkOption {
      type = lib.types.attrs;
      default = fenix.combine [
        (
          if config.stdenv.hostPlatform.isDarwin
          then darwinToolchain
          else fenixDefault.toolchain
        )
        wasmRustStd
      ];
    };
  };
}
