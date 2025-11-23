{
  description = "Convert your favorite images and wallpapers with your favorite color palettes/themes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.maybeMissing ./src/palettes)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = [
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          meta = {
            description = "Convert your favorite images and wallpapers with your favorite color palettes/themes";
            homepage = "https://github.com/doprz/dipc";
            license = with lib.licenses; [
              asl20
              mit
            ];
            # maintainers = with lib.maintainers; [ doprz ];
            platforms = lib.platforms.unix ++ lib.platforms.darwin;
            mainProgram = "dipc";
          };
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency artifacts from above.
        dipc = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit dipc;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          dipc-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          dipc-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          dipc-fmt = craneLib.cargoFmt { inherit src; };
          dipc-audit = craneLib.cargoAudit { inherit src advisory-db; };
          dipc-deny = craneLib.cargoDeny { inherit src; };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `dipc` if you do not want
          # the tests to run twice
          dipc-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          default = dipc;
          inherit dipc;
        };

        apps.default = flake-utils.lib.mkApp { drv = dipc; };

        formatter = pkgs.nixfmt-rfc-style;

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = with pkgs; [ rust-analyzer ];
        };
      }
    );
}
