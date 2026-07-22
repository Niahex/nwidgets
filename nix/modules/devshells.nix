{ inputs, ... }:
{
  perSystem =
    { pkgs, system, ... }:
    let
      mkNwidgets = import ../toolchain.nix { inherit inputs; };
      nwidgets = mkNwidgets pkgs;

      rustBin = inputs.rust-overlay.lib.mkRustBin { } pkgs;
      rustToolchain = rustBin.fromRustupToolchainFile ../../rust-toolchain.toml;

      baseEnv =
        (nwidgets.overrideAttrs (attrs: {
          passthru.env = attrs.env;
        })).env;
    in
    {
      devShells.default = (pkgs.mkShell.override { inherit (nwidgets) stdenv; }) {
        name = "nwidgets-dev";
        inputsFrom = [ nwidgets ];

        packages = with pkgs; [
          rustToolchain
          cargo-nextest
          cargo-machete
        ];

        env = builtins.removeAttrs baseEnv [
          "CARGO_PROFILE"
          "TARGET_DIR"
        ] // {
          FONTCONFIG_FILE = pkgs.makeFontsConf {
            fontDirectories = [
              "${pkgs.nerd-fonts.ubuntu}/share/fonts"
              "${pkgs.nerd-fonts.ubuntu-mono}/share/fonts"
              "${pkgs.nerd-fonts.ubuntu-sans}/share/fonts"
            ];
          };
        };

        shellHook = ''
          echo "🪟 nwidgets dev shell"
          echo "   Rust: $(rustc --version)"
          echo "   Cargo: $(cargo --version)"
        '';
      };
    };
}
