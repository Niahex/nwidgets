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
        # Overlays and package set
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        # Rust toolchain configuration
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = ["rust-src"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        
        # When filtering sources, we want to allow assets other than .rs files
        unfilteredRoot = ./.; # The original, unfiltered source
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
                "xml"
              ]
            ) unfilteredRoot)
            # Assets folder for icons
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        # Dependencies for building the application
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
          glib # Ajout explicite de glib
          webkitgtk_6_0 # For webkit6 - GTK4 version
          libsoup_3 # For webkit6 networking
          glib-networking # For TLS support
          gsettings-desktop-schemas # For WebKit settings
          cacert # SSL certificates
          gnutls # TLS library
          atk # Accessibility toolkit
          at-spi2-atk # AT-SPI bridge
          gtk4-layer-shell # For GTK4 layer shell
          openblas # For whisper/transcribe-rs
          llvmPackages.libclang.lib
          vulkan-headers
          vulkan-loader
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
          # GIO_USE_TLS = "gnutls"; # Souvent inutile si glib-networking est bien chargé, mais on peut le laisser si nécessaire
          SSL_CERT_FILE = "/nix/var/nix/profiles/system/etc/ssl/certs/ca-bundle.crt";
          NIX_SSL_CERT_FILE = "/nix/var/nix/profiles/system/etc/ssl/certs/ca-bundle.crt";
          # Ensure pkg-config finds openblas
          PKG_CONFIG_PATH = "${pkgs.openblas}/lib/pkgconfig";
          # For bindgen (used by whisper-rs-sys)
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.clang}/include";
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

          # CORRECTION : Utilisation de GIO_EXTRA_MODULES au lieu de GIO_MODULE_DIR
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

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies);
          FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = buildInputs;};

          # CORRECTION : Même chose pour le shell de dev
          shellHook = ''
            export GIO_EXTRA_MODULES="${pkgs.glib-networking}/lib/gio/modules:$GIO_EXTRA_MODULES"
            echo "[🦀 Rust $(rustc --version)] - Ready to develop nwidgets!"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
