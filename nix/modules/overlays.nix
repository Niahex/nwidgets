{ inputs, ... }:
{
  flake.overlays.default =
    final: _:
    let
      mkNwidgets = import ../toolchain.nix { inherit inputs; };
    in
    {
      nwidgets = mkNwidgets final;
    };
}
