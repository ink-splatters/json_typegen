top @ {lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit
      (pkgs.llvmPackages_latest)
      bintools
      libcxx
      ;
    stdenv = pkgs.llvmPackages_latest.libcxxStdenv;
    clang = stdenv.cc;
    wasmBintools = pkgs.llvmPackages_latest.bintools-unwrapped;

    mkFlags = flags: lib.concatStringsSep " " (map (x: "-C ${x}") flags);

    benchmarkFixtureFiles = map toString [
      (top.config.src + "/json_typegen_shared/benches/fixtures/magic_card_list.json")
      (top.config.src + "/json_typegen_shared/benches/fixtures/zalando_article.json")
    ];
    cargoSrc = lib.cleanSourceWith {
      src = top.config.src;
      filter = path: type:
        config.craneLib.filterCargoSources path type
        || builtins.elem (toString path) benchmarkFixtureFiles;
    };
    wasmPackageFiles = map toString [
      (top.config.src + "/json_typegen_wasm/LICENSE")
      (top.config.src + "/json_typegen_wasm/README.md")
    ];
    wasmSrc = lib.cleanSourceWith {
      src = top.config.src;
      filter = path: type:
        config.craneLib.filterCargoSources path type
        || builtins.elem (toString path) wasmPackageFiles;
    };

    flags = [
      "linker=${clang}/bin/cc"
      "link-args=-fuse-ld=lld"
      "embed-bitcode=yes"
      "lto=thin"
    ];

    # CFLAGS = "-O3 -pipe";
    # CXXFLAGS = "-O3 -pipe";
    # LDFLAGS = "-fuse-ld=lld";
    # mkCFlagsNative = flags: "${flags} -mcpu=${top.config.native}";

    mkCommonArgs = args @ {flags, ...}:
      {
        src = cargoSrc;
        strictDeps = true;
        enableParallelBuilding = true;
        RUSTFLAGS = "-Zdylib-lto " + (mkFlags flags);

        buildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
          libcxx
        ];

        nativeBuildInputs =
          [
            clang
            bintools
          ]
          ++ lib.optionals stdenv.hostPlatform.isDarwin [
            pkgs.apple-sdk_15
          ];
        # inherit CFLAGS CXXFLAGS LDFLAGS;
      }
      // (builtins.removeAttrs args ["flags"]);
  in {
    options = {
      stdenv = lib.mkOption {
        type = lib.types.attrs;
        default = stdenv;
      };

      commonArgs = lib.mkOption {
        type = lib.types.attrs;
        default = mkCommonArgs {inherit flags;};
      };

      commonArgsNative = lib.mkOption {
        type = lib.types.attrs;

        default = mkCommonArgs {
          flags = flags ++ ["target-cpu=${top.config.native}"];
          NIX_ENFORCE_NO_NATIVE = 0;

          # CFLAGS = mkCFlagsNative CFLAGS;
          # CXXFLAGS = mkCFlagsNative CXXFLAGS;
        };
      };

      commonArgsWeb = lib.mkOption {
        type = lib.types.attrs;
        default = {
          src = wasmSrc;
          strictDeps = true;
          enableParallelBuilding = true;
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${wasmBintools}/bin/wasm-ld";
          nativeBuildInputs = with pkgs; [
            wasmBintools
            binaryen
            wasm-bindgen-cli_0_2_121
            wasm-pack
            writableTmpDirAsHomeHook
          ];
        };
      };
    };
  };
}
