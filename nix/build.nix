{
  pkgs,
  lib,
  stdenv,

  crane,
  rustToolchain,

  alsa-lib,
  cmake,
  fontconfig,
  freetype,
  glib,
  libdrm,
  libgbm,
  libglvnd,
  libva,
  libxcomposite,
  libxdamage,
  libxext,
  libxfixes,
  libxkbcommon,
  libxrandr,
  libx11,
  libxcb,
  openssl,
  pkg-config,
  vulkan-loader,
  wayland,
  zlib,
  zstd,

  withGLES ? false,
  profile ? "release",
}:
assert withGLES -> stdenv.hostPlatform.isLinux;

let
  craneLib = crane.overrideToolchain rustToolchain;
  gpu-lib = if withGLES then libglvnd else vulkan-loader;

  src = lib.cleanSourceWith {
    src = ../.;
    filter = path: type:
      let
        relPath = lib.removePrefix (toString ../. + "/") (toString path);
        topLevel = builtins.head (lib.splitString "/" relPath);
      in
      builtins.elem topLevel [
        "crates"
        "Cargo.toml"
        "Cargo.lock"
        ".cargo"
        "rust-toolchain.toml"
        "rustfmt.toml"
        "clippy.toml"
      ];
    name = "source";
  };

  commonArgs = {
    pname = "nwidgets";
    version = "0.1.0";
    inherit src;

    cargoLock = ../Cargo.lock;

    nativeBuildInputs = [
      cmake
      pkg-config
    ];

    buildInputs = [
      fontconfig
      pkgs.nerd-fonts.ubuntu
      pkgs.nerd-fonts.ubuntu-mono
      pkgs.nerd-fonts.ubuntu-sans
      freetype
      openssl
      zlib
      zstd
    ] ++ lib.optionals stdenv.hostPlatform.isLinux [
      alsa-lib
      glib
      libva
      libxkbcommon
      wayland
      gpu-lib
      libglvnd
      libx11
      libxcb
      libdrm
      libgbm
      libxcomposite
      libxdamage
      libxext
      libxfixes
      libxrandr
    ];

    cargoExtraArgs = "--workspace --locked";

    env = {
      ZSTD_SYS_USE_PKG_CONFIG = true;
      FONTCONFIG_FILE = lib.optionalString stdenv.hostPlatform.isLinux (
        pkgs.makeFontsConf {
          fontDirectories = [
            "${pkgs.nerd-fonts.ubuntu}/share/fonts"
            "${pkgs.nerd-fonts.ubuntu-mono}/share/fonts"
            "${pkgs.nerd-fonts.ubuntu-sans}/share/fonts"
          ];
        }
      );
      CARGO_PROFILE = profile;
      TARGET_DIR = "target/" + (if profile == "dev" then "debug" else profile);
    };

    dontPatchELF = stdenv.hostPlatform.isLinux;
    doCheck = false;

    cargoVendorDir = craneLib.vendorCargoDeps {
      inherit src;
      cargoLock = ../Cargo.lock;
    };
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  lib.recursiveUpdate commonArgs {
    inherit cargoArtifacts;

    passthru = {
      inherit craneLib commonArgs cargoArtifacts;
    };

    dontUseCmakeConfigure = true;

    meta = {
      description = "nwidgets: Wayland widget system built with GPUI";
      homepage = "https://github.com/Niahex/nwidgets";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
    };
  }
)
