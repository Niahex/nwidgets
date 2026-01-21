{
  description = "nwidgets - A GPUI-based application launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
      "https://cache.nixos.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
    ];
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        # Overlays and package set
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        # Rust toolchain configuration
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = ["rust-src"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (
                file:
                  pkgs.lib.any file.hasExt [
                    "svg"
                  ]
              )
              unfilteredRoot)
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        # CEF Configuration
        cefVersion = "143.0.14+gdd46a37+chromium-143.0.7499.193";
        cefPlatform = "linux64";
        cefSrc = pkgs.fetchurl {
          url = "https://cef-builds.spotifycdn.com/cef_binary_${pkgs.lib.strings.escapeURL cefVersion}_${cefPlatform}_minimal.tar.bz2";
          name = "cef_binary_${pkgs.lib.strings.escapeURL cefVersion}_${cefPlatform}_minimal.tar.bz2";
          hash = "sha256-BPlAGOHOxIkgpX+yMHUDxy+xk2FXgyXf1Ex9Uibn7cM=";
        };

        cefDeps = with pkgs; [
          glib
          nss
          nspr
          at-spi2-atk
          libdrm
          expat
          mesa
          alsa-lib
          dbus
          cups
          libxkbcommon
          pango
          cairo
          udev
          xorg.libX11
          xorg.libXcomposite
          xorg.libXdamage
          xorg.libXext
          xorg.libXfixes
          xorg.libXrandr
          xorg.libXcursor
          xorg.libXrender
          xorg.libXScrnSaver
          xorg.libXtst
          xorg.libxcb
          libglvnd
          vulkan-loader
          libayatana-appindicator
          gtk3
        ];

        cefAssets =
          pkgs.runCommand "cef-assets" {
            nativeBuildInputs = [pkgs.autoPatchelfHook];
            buildInputs = cefDeps;
          } ''
            mkdir -p $out
            mkdir temp
            tar -xf ${cefSrc} --strip-components=1 -C temp

            # Mimic extract_target_archive from download-cef/src/lib.rs
            # 1. Move everything from Release to $out
            cp -r temp/Release/* $out/

            # 2. Move everything from Resources to $out
            cp -r temp/Resources/* $out/

            # 3. Move include and cmake and libcef_dll to $out (needed for build)
            cp -r temp/include $out/
            cp -r temp/cmake $out/
            cp -r temp/libcef_dll $out/
            cp temp/CMakeLists.txt $out/

            # Generate archive.json which is required by cef-rs
            echo '{"type":"minimal","name":"cef_binary_${cefVersion}_${cefPlatform}_minimal.tar.bz2","sha1":""}' > $out/archive.json

            # Remove chrome-sandbox before patching (it requires setuid and can't be patched)
            rm -f $out/chrome-sandbox

            # Patch binaries in $out
            autoPatchelf $out
          '';

        # Dependencies for building the application
        buildInputs = with pkgs;
          [
            wayland
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools
            mesa
            xorg.libxcb
            xorg.libX11
            libxkbcommon
            fontconfig
            dbus
            openssl
            freetype
            expat
            libnotify
            alsa-lib
            udev
            pipewire
          ]
          ++ cefDeps;

        # Dependencies needed only at runtime
        runtimeDependencies = with pkgs;
          [
            wayland
            vulkan-loader
            mesa
            libxkbcommon
            wayland
          ]
          ++ cefDeps;

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
          autoPatchelfHook
          clang
          cmake
          ninja
          rustPlatform.bindgenHook
          mold # Fast linker to handle long argument lists
        ];

        envVars = {
          RUST_BACKTRACE = "full";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          CEF_PATH = cefAssets;
          # Use mold linker with rpaths for AMD/NVIDIA Vulkan support
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold -C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath [pkgs.vulkan-loader pkgs.wayland]} -C link-arg=-Wl,-rpath,${cefAssets} -C link-arg=-L${cefAssets}";
          NIX_LDFLAGS = "-rpath ${pkgs.lib.makeLibraryPath [pkgs.vulkan-loader pkgs.wayland]}";
        };

        # Build artifacts
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs nativeBuildInputs;
          env = envVars;
        };

        # Application package definition
        nwidgets = craneLib.buildPackage {
          inherit src cargoArtifacts buildInputs nativeBuildInputs runtimeDependencies;
          env = envVars;
          pname = "nwidgets";
          version = "0.1.0";

          # Prevent nix from removing "unused" wayland/vulkan rpaths
          dontPatchELF = true;

          postFixup = ''
            # Copy assets to the output
            mkdir -p $out/share/nwidgets
            cp -r ${src}/assets $out/share/nwidgets/

            # Copy CEF runtime assets to bin (where the executable is)
            # We copy everything from cefAssets except include/cmake/libcef_dll
            find ${cefAssets} -maxdepth 1 -type f -exec cp {} $out/bin/ \;
            cp -r ${cefAssets}/locales $out/bin/ || true

            wrapProgram $out/bin/nwidgets \
              --prefix LD_LIBRARY_PATH : /run/opengl-driver/lib:${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}:${cefAssets} \
              --set NWIDGETS_ASSETS_DIR $out/share/nwidgets/assets \
              --set CEF_PATH ${cefAssets} \
              --run 'export VK_ICD_FILENAMES=$(find /run/opengl-driver/share/vulkan/icd.d -name "*_icd.*.json" 2>/dev/null | tr "\n" ":" | sed "s/:$//")' \
              --run 'if lspci 2>/dev/null | grep -qi nvidia; then export __GL_THREADED_OPTIMIZATIONS=1 __GL_YIELD=USLEEP; elif lspci 2>/dev/null | grep -qi amd; then export MESA_GLTHREAD=true RADV_PERFTEST=gpl; fi'
          '';
        };

        # Development shell tools
        devTools = with pkgs; [
          rust-analyzer
          rustToolchain
          cargo-watch
          cargo-edit
          cargo-audit
          cargo-machete
          cargo-bloat
          cargo-flamegraph
          bacon
          libnotify
        ];
      in {
        packages = {
          default = nwidgets;
          inherit nwidgets;
        };

        checks = {
          inherit nwidgets;

          nwidgets-clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts buildInputs nativeBuildInputs;
            env = envVars;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          nwidgets-fmt = craneLib.cargoFmt {inherit src;};
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [nwidgets];
          nativeBuildInputs = devTools;
          env = envVars;

          # Vulkan libs first, then system GL, then CEF
          LD_LIBRARY_PATH = "/run/opengl-driver/lib:${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}:${cefAssets}";
          __EGL_VENDOR_LIBRARY_FILENAMES = "/run/opengl-driver/share/glvnd/egl_vendor.d/50_mesa.json:/run/opengl-driver/share/glvnd/egl_vendor.d/10_nvidia.json";
          FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = buildInputs;};

          # Force GPUI to use Vulkan backend (auto-detects AMD/NVIDIA/Intel)
          GPUI_BACKEND = "vulkan";

          shellHook = ''
            echo "[🦀 Rust $(rustc --version)] - Ready to develop nwidgets!"
            
            # Auto-detect available Vulkan ICDs
            export VK_ICD_FILENAMES=$(find /run/opengl-driver/share/vulkan/icd.d -name "*_icd.*.json" 2>/dev/null | tr '\n' ':' | sed 's/:$//')
            
            # GPU-specific optimizations
            if lspci 2>/dev/null | grep -qi nvidia; then
              export __GL_THREADED_OPTIMIZATIONS=1
              export __GL_YIELD=USLEEP
              echo "GPU: NVIDIA detected - threaded optimizations enabled"
            elif lspci 2>/dev/null | grep -qi amd; then
              export MESA_GLTHREAD=true
              export RADV_PERFTEST=gpl
              echo "GPU: AMD detected - RADV optimizations enabled"
            else
              echo "GPU: Auto-detected (AMD/NVIDIA/Intel)"
            fi
            
            echo "Vulkan ICD: $VK_ICD_FILENAMES"
            echo "Available Vulkan devices:"
            vulkaninfo --summary 2>/dev/null | grep -A 2 "GPU" || echo "  Run 'vulkaninfo' for details"
            export CEF_PATH="${cefAssets}"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
