{
  description = "nwidgets - A GTK4-based application launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
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
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = ["rust-src"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (
                file:
                  pkgs.lib.any file.hasExt [
                    "scss"
                    "svg"
                  ]
              )
              unfilteredRoot)
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        buildInputs = with pkgs; [
          fontconfig
          dbus
          openssl
          freetype
          expat
          nerd-fonts.ubuntu-mono
          nerd-fonts.ubuntu-sans
          nerd-fonts.ubuntu
          noto-fonts-emoji
          libnotify
          alsa-lib # For audio capture (cpal/vosk)
          udev # For libinput (hotkey detection)
          gtk4 # For GTK4 webview
          gtk3 # For webkit2gtk compatibility
          glib # Ajout explicite de glib
          webkitgtk_4_1 # For webkit2gtk - GTK3/4 version
          libsoup_3 # For webkit6 networking
          glib-networking # For TLS support
          gsettings-desktop-schemas # For WebKit settings
          cacert # SSL certificates
          gnutls # TLS library
          atk # Accessibility toolkit
          at-spi2-atk # AT-SPI bridge
          gtk4-layer-shell # For GTK4 layer shell
          llvmPackages.libclang.lib
          vulkan-headers
          vulkan-loader
          onnxruntime # For transcribe-rs
          nss
          nspr
          libdrm
          libxkbcommon
          libgbm
          pango
          cairo
          cups
          libGL
          systemdLibs
          xorg.libxcb
          xorg.libX11
          xorg.libXcomposite
          xorg.libXdamage
          xorg.libXext
          xorg.libXfixes
          xorg.libXrandr
          xorg.libxshmfence
          # # ROCm pour GPU AMD (whisper.cpp hipBLAS)
          # rocmPackages.clr
          # rocmPackages.hipblas
          # rocmPackages.rocblas
          # rocmPackages.rocm-runtime
        ];

        # Dependencies needed only at runtime
        runtimeDependencies = with pkgs; [
          wayland
          vulkan-loader
          wtype # For text injection in dictation
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
          autoPatchelfHook
          clang # For building whisper-rs/transcribe-rs C++ parts
          mold # Faster linker (optional but good)
          rustPlatform.bindgenHook
          cmake
          shaderc # For glslc (Vulkan shaders)
        ];

        envVars = {
          RUST_BACKTRACE = "full";
          SSL_CERT_FILE = "/nix/var/nix/profiles/system/etc/ssl/certs/ca-bundle.crt";
          NIX_SSL_CERT_FILE = "/nix/var/nix/profiles/system/etc/ssl/certs/ca-bundle.crt";
          # Ensure pkg-config finds openblas
          PKG_CONFIG_PATH = "${pkgs.openblas}/lib/pkgconfig";
          # For bindgen (used by whisper-rs-sys)
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.clang}/include";
          # For ort-sys (ONNX Runtime) - skip download and use system library
          ORT_SKIP_DOWNLOAD = "1";
          ORT_LIB_LOCATION = "${pkgs.onnxruntime}";
          # ONNX Runtime optimizations for i9-9900K (AVX2, FMA support)
          ORT_NUM_THREADS = "16"; # i9-9900K has 16 threads
          ORT_OPTIMIZATION_LEVEL = "3"; # Maximum optimization
          # CPU-specific optimizations
          RUSTFLAGS = "-C target-cpu=native -C opt-level=3 -L";
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

          postFixup = ''
            wrapProgram $out/bin/nwidgets \
              --prefix GIO_EXTRA_MODULES : "${pkgs.glib-networking}/lib/gio/modules" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath buildInputs}"
          '';
        };

        # Development shell tools
        devTools = with pkgs; [
          rust-analyzer
          rustToolchain
          cargo-watch
          cargo-edit
          cargo-audit
          bacon
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

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}";
          FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = buildInputs;};

          shellHook = ''
            export GIO_EXTRA_MODULES="${pkgs.glib-networking}/lib/gio/modules:$GIO_EXTRA_MODULES"

            # ONNX Runtime optimizations for i9-9900K
            export ORT_NUM_THREADS=16
            export ORT_INTRA_OP_NUM_THREADS=16
            export ORT_INTER_OP_NUM_THREADS=1
            export ORT_OPTIMIZATION_LEVEL=3

            # CPU optimizations (AVX2, FMA for i9-9900K)
            export RUSTFLAGS="-C target-cpu=native -C opt-level=3"

            echo "[🦀 Rust $(rustc --version)] - Ready to develop nwidgets!"
            echo "⚡ CPU: i9-9900K optimizations enabled (16 threads, AVX2/FMA)"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
