{ sources ? import ./nix/sources.nix }:
import sources.nixpkgs {
  overlays = [
    (self: super: {
      mars = super.callPackage ./nix/mars.nix {};
    })
  ];
}