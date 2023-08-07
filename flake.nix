{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};
      in
    {
      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
          doCheck = true;
          #buildInputs = [];
          #nativeBuildInputs = [];
      };
    });
}
