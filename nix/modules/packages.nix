{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      lib,
      system,
      ...
    }:
    let
      mkNwidgets = import ../toolchain.nix { inherit inputs; };
      nwidgets = mkNwidgets pkgs;
    in
    {
      packages = {
        default = nwidgets;
        debug = nwidgets.override { profile = "dev"; };
      };
    };
}
