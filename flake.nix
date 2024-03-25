{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, gitignore }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      libraries = with pkgs; [ hunspell.dev libclang.lib stdenv.cc.cc.lib ];

      nativeBuildInputs = with pkgs; [
        rustToolchain
        pkg-config
        act
        rust-analyzer

        cargo-workspaces
        cargo-feature
      ];
      buildInputs = with pkgs; [ openssl.dev hunspell.dev libclang.lib ];
    in
    {
      formatter = pkgs.nixpkgs-fmt;

      packages = rec {
        default = git-leave;
        git-leave = pkgs.callPackage ./package.nix { inherit gitignore; };
      };
      apps = rec {
        default = git-leave;
        git-leave = flake-utils.lib.mkApp { drv = self.packages.${system}.git-leave; };
      };

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;

        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

        RUST_LOG = "lspelling_lsp=debug,lspelling_wordc=debug,info";

        HUNSPELL_DICT = "${pkgs.hunspellDicts.en_US-large}/share/hunspell/en_US";
      };
    }
  );
}
