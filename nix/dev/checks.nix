top @ {inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) craneLib commonArgs cargoArtifacts;
    workspace = craneLib.crateNameFromCargoToml {
      cargoToml = top.config.src + "/json_typegen_cli/Cargo.toml";
    };
    checkIdentity = {
      pname = "json-typegen-workspace";
      inherit (workspace) version;
    };
    sourceArgs =
      checkIdentity
      // {
        inherit (commonArgs) src;
      };
    workspaceArgs =
      commonArgs
      // checkIdentity
      // {
        cargoExtraArgs = "--workspace --all-features";
      };
  in {
    checks = {
      inherit
        (config.packages)
        json-typegen
        json-typegen-demo
        json-typegen-wasm
        json-typegen-web
        ;

      cargo-audit = craneLib.cargoAudit (
        sourceArgs
        // {
          inherit (inputs) advisory-db;
        }
      );

      cargo-clippy = craneLib.cargoClippy (
        workspaceArgs
        // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        }
      );

      cargo-doc = craneLib.cargoDoc (
        workspaceArgs
        // {
          inherit cargoArtifacts;
        }
      );

      cargo-deny = craneLib.cargoDeny (
        sourceArgs
        // {
          cargoDenyChecks = "bans licenses sources";
        }
      );

      cargo-fmt = craneLib.cargoFmt sourceArgs;

      cargo-nextest = craneLib.cargoNextest (
        workspaceArgs
        // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
          cargoNextestPartitionsExtraArgs = "--no-tests=pass";
        }
      );

      cargo-udeps = craneLib.mkCargoDerivation (
        workspaceArgs
        // {
          inherit cargoArtifacts;
          pnameSuffix = "-udeps";
          buildPhaseCargoCommand = "cargo udeps --locked --workspace --all-targets --all-features";
          doInstallCargoArtifacts = false;
          nativeBuildInputs =
            (commonArgs.nativeBuildInputs or [])
            ++ [
              pkgs.cargo-udeps
            ];
        }
      );
    };
  };
}
