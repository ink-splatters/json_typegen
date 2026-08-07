{
  perSystem = {
    config,
    inputs',
    ...
  }: let
    inherit (config) pre-commit craneLib commonArgs commonArgsWeb;
    inherit (inputs'.fenix.packages.complete) rust-src;
  in {
    devShells.default = craneLib.devShell (
      (builtins.removeAttrs commonArgs ["src"])
      // {
        inherit (commonArgsWeb) CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER;
        packages = pre-commit.settings.enabledPackages ++ commonArgsWeb.nativeBuildInputs;

        shellHook = ''
          export RUST_SRC_PATH="${rust-src}/lib/rustlib/src/rust/library"
          ${pre-commit.installationScript}
        '';
      }
    );
  };
}
