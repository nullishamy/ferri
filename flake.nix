{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;

          overlays = [
            (import inputs.rust-overlay)
          ];
        };

        devShells.default = pkgs.mkShell {
          env = {
            DATABASE_URL = "sqlite:ferri.db";
          };
          
          packages = with pkgs; [
            sqlx-cli
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [  ];
            })
          ]; 
        };
      };
    };
}
