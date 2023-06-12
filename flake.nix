{
  description = "Dev shell the project";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust_toolchain = fenix.packages.${system}.complete.toolchain;
      in with pkgs; {
        nixpkgs.overlays = [ fenix.overlays.complete ];
        devShells.default = mkShell rec {


          nativeBuildInputs =  [
            pkg-config
            cmake
            pkg-config
            freetype
            expat
            fontconfig
          ];

          buildInputs = [
            rust_toolchain
            lldb_15
            sccache
            sccache
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            libxkbcommon
            wayland
            wasm-bindgen-cli
          ];
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
        };
      });
}

