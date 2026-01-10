{
  description = "Hongdown flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        rust = pkgs.rust-bin.stable.latest.default;

        # Parse the current package version directly from the `Cargo.toml` file.
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        inherit (cargoToml.package) version;
      in {
        # Run `nix develop` to get a reproducible dev shell
        devShells.default = pkgs.mkShell {
          buildInputs = [rust];
        };
        # Build nix packages with `nix build`
        packages = rec {
          default = hongdown;
          hongdown = pkgs.callPackage ./nix/hongdown.nix {inherit version;};
        };
        # Run the program with `nix run`
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}
