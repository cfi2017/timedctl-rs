{
  description = "timedctl-rs nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      pre-commit-hooks,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
        formatter =
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          pkgs.nixfmt-rfc-style;
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              statix.enable = false;
              nixfmt-rfc-style.enable = true;
            };
          };
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          NIX_CONFIG = "experimental-features = nix-command flakes";
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];
        };

        packages.default =
          (pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          }).buildRustPackage
            {
              pname = "timedctl";
              version = "0.1.0";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              nativeBuildInputs = [ pkgs.pkg-config ];
              # buildInputs = [ pkgs.openssl ];
            };

        formatter = formatter;
        checks = checks;
      }
    );
}
