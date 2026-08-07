top @ {lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) stdenv;
    wasmPackage = config.packages.json-typegen-wasm;
    webPackage = builtins.fromJSON (builtins.readFile (top.config.src + "/json_typegen_web/package.json"));

    webSrc = lib.cleanSourceWith {
      src = top.config.src;
      filter = path: _type: let
        root = toString top.config.src;
        path' = toString path;
        relative = lib.removePrefix "${root}/" path';
        isWeb = relative == "json_typegen_web" || lib.hasPrefix "json_typegen_web/" relative;
        isGenerated =
          relative
          == "json_typegen_web/dist"
          || lib.hasPrefix "json_typegen_web/dist/" relative
          || relative == "json_typegen_web/node_modules"
          || lib.hasPrefix "json_typegen_web/node_modules/" relative;
      in
        path' == root || (isWeb && !isGenerated);
    };

    nodeModules = stdenv.mkDerivation {
      pname = "json-typegen-web-node-modules";
      inherit (webPackage) version;
      src = webSrc;

      strictDeps = true;
      nativeBuildInputs = [
        pkgs.bun
        pkgs.writableTmpDirAsHomeHook
      ];

      dontConfigure = true;
      buildPhase = ''
        runHook preBuild

        mkdir -p json_typegen_wasm/pkg
        cp -R ${wasmPackage}/. json_typegen_wasm/pkg/

        pushd json_typegen_web
        export BUN_INSTALL_CACHE_DIR=$(mktemp -d)
        bun install \
          --cpu="*" \
          --os="*" \
          --frozen-lockfile \
          --ignore-scripts \
          --no-progress
        rm -rf node_modules/json_typegen_wasm
        popd

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall

        mkdir -p $out
        cp -R json_typegen_web/node_modules $out/

        runHook postInstall
      '';

      dontFixup = true;
      outputHash = "sha256-vrlQKMoqP7klcFBCD0NdrbBnP9IFcZbDAnURGRlgAIA=";
      outputHashAlgo = "sha256";
      outputHashMode = "recursive";
    };
  in {
    packages.json-typegen-web = stdenv.mkDerivation {
      pname = "json-typegen-web";
      inherit (webPackage) version;
      src = webSrc;

      strictDeps = true;
      nativeBuildInputs = [pkgs.bun];

      configurePhase = ''
        runHook preConfigure

        rm -rf json_typegen_web/node_modules
        cp -R ${nodeModules}/node_modules json_typegen_web/
        chmod -R u+w json_typegen_web/node_modules
        mkdir json_typegen_web/node_modules/json_typegen_wasm
        cp -R ${wasmPackage}/. json_typegen_web/node_modules/json_typegen_wasm/

        runHook postConfigure
      '';

      buildPhase = ''
        runHook preBuild

        pushd json_typegen_web
        bun run build
        popd

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall

        mkdir -p $out
        cp -R json_typegen_web/dist/. $out/

        runHook postInstall
      '';

      doInstallCheck = true;
      installCheckPhase = ''
        test -f $out/index.html
        test -n "$(find $out -type f -name '*.wasm' -print -quit)"
      '';
    };
  };
}
