top @ {lib, ...}: {
  perSystem = {config, ...}: let
    inherit (config) craneLib commonArgs commonArgsNative commonArgsWeb;
    inherit
      (craneLib.crateNameFromCargoToml {
        cargoToml = top.config.src + "/json_typegen_cli/Cargo.toml";
      })
      version
      ;
  in {
    options = {
      cargoArtifacts = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "json-typegen-workspace";
            inherit version;
            cargoExtraArgs = "--workspace --all-features";
          });
      };
      cargoArtifactsNative = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgsNative
          // {
            pname = "json-typegen-native";
            inherit version;
            cargoExtraArgs = "-p json_typegen_cli";
          });
      };
      cargoArtifactsWasm = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgsWeb
          // {
            pname = "json-typegen-wasm";
            inherit version;
            cargoExtraArgs = "-p json_typegen_wasm --all-features";
            doCheck = false;
          });
      };
    };
  };
}
