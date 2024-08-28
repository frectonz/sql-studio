{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          (pkgs.lib.hasSuffix "\.css" path) ||
          (pkgs.lib.hasSuffix "\.js" path) ||
          (pkgs.lib.hasSuffix "\.svg" path) ||
          (pkgs.lib.hasSuffix "\.sqlite3" path) ||
          (craneLib.filterCargoSources path type)
        ;
      };
      commonArgs = {
        inherit src;
        buildInputs = [ pkgs.git ];
      };

      ui = pkgs.buildNpmPackage {
        pname = "ui";
        version = "0.0.0";
        src = ./ui;
        npmDepsHash = "sha256-kGukH0PKF7MtIO5UH+55fddj6Tv2dNLmOC6oytEhP3c=";
        installPhase = ''
          cp -pr --reflink=auto -- dist "$out/"
        '';
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      bin = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        preBuild = ''
          cp -pr --reflink=auto -- ${ui} ui/dist
        '';
      });

      docker = pkgs.dockerTools.streamLayeredImage {
        name = "sql-studio";
        tag = "latest";
        created = "now";
        config.Cmd = [
          "${bin}/bin/sql-studio"
          "--no-browser"
          "--no-shutdown"
          "--address=0.0.0.0:3030"
          "sqlite"
          "preview"
        ];
        config.Expose = "3030";
      };
    in
    {
      packages = {
        default = bin;
        docker = docker;
        stable = pkgs.callPackage ./package.nix { };
      };

      devShells.default = pkgs.mkShell {
        buildInputs = [
          pkgs.bacon
          pkgs.emmet-ls
          pkgs.cargo-dist
          pkgs.cargo-watch
          pkgs.rust-analyzer
          pkgs.cargo-outdated
          rustToolchain

          pkgs.nodejs
          pkgs.nodePackages.typescript-language-server
          pkgs.nodePackages.vscode-langservers-extracted
          pkgs.nodePackages."@tailwindcss/language-server"

          pkgs.httpie
          pkgs.sqlite
        ];
      };

      formatter = pkgs.nixpkgs-fmt;
    }
  );
}
