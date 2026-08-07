top: {
  perSystem = {config, ...}: let
    inherit
      (config)
      craneLib
      commonArgs
      commonArgsNative
      commonArgsWeb
      cargoArtifacts
      cargoArtifactsNative
      cargoArtifactsWasm
      ;

    crateVersion = crate:
      (craneLib.crateNameFromCargoToml {
        cargoToml = top.config.src + "/${crate}/Cargo.toml";
      }).version;
  in {
    packages = {
      json-typegen = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "json_typegen";
          version = crateVersion "json_typegen_cli";
          cargoExtraArgs = "-p json_typegen_cli";
          meta.mainProgram = "json_typegen";
        });

      json-typegen-native = craneLib.buildPackage (commonArgsNative
        // {
          cargoArtifacts = cargoArtifactsNative;
          pname = "json_typegen-native";
          version = crateVersion "json_typegen_cli";
          cargoExtraArgs = "-p json_typegen_cli";
          meta.mainProgram = "json_typegen";
        });

      json-typegen-demo = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "json_typegen_demo";
          version = crateVersion "json_typegen_demo";
          cargoExtraArgs = "-p json_typegen_demo";
          meta.mainProgram = "json_typegen_demo";
        });

      json-typegen-wasm = craneLib.mkCargoDerivation (commonArgsWeb
        // {
          cargoArtifacts = cargoArtifactsWasm;
          pname = "json_typegen_wasm";
          version = crateVersion "json_typegen_wasm";
          doCheck = false;
          doInstallCargoArtifacts = false;
          buildPhaseCargoCommand = ''
            wasm-pack build json_typegen_wasm \
              --target web \
              --out-dir pkg \
              --mode no-install \
              -- \
              --locked \
              --offline
          '';
          installPhaseCommand = ''
            mkdir -p "$out"
            cp -R json_typegen_wasm/pkg/. "$out/"
            test -f "$out/json_typegen_wasm.js"
            test -f "$out/json_typegen_wasm_bg.wasm"
            test -f "$out/package.json"
          '';
        });
    };
  };
}
