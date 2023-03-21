{
  description = "Write your own river layout generator in lua";

  outputs = { self, nixpkgs }:
    let

      # to work with older version of flakes
      lastModifiedDate = self.lastModifiedDate or self.lastModified or "19700101";

      # Generate a user-friendly version number.
      version = "${builtins.substring 0 8 lastModifiedDate}-${self.shortRev or "dirty"}";

      # System types to support.
      supportedSystems = [ "x86_64-linux" ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; overlays = [ self.overlay ]; });

    in
    {

      # A Nixpkgs overlay.
      overlay = final: prev: with final; {

        river-luatile = pkgs.rustPlatform.buildRustPackage {
          pname = "river-luatile";
          inherit version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "river-layout-toolkit-0.1.0" = "sha256-ctI0n4Jwf1oMjmGf1Lsko1CHfOQTTphCH4Ona3aHXj4=";
              "wayrs-client-0.1.0" = "sha256-94URrCTfBfub2TE00/9fmbs1opnIBsBsa0m3oIJRtm4=";
            };
          };

          buildInputs = with pkgs; [luajit ];
          nativeBuildInputs = with pkgs; [pkg-config ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      };

      # Provide some binary packages for selected system types.
      packages = forAllSystems (system:
        {
          inherit (nixpkgsFor.${system}) river-luatile;
        });

      # The default package for 'nix build'. This makes sense if the
      # flake provides only one package or there is a clear "main"
      # package.
      defaultPackage = forAllSystems (system: self.packages.${system}.river-luatile);

      # Provide a 'nix develop' environment for interactive hacking.
      devShell = forAllSystems (system: self.packages.${system}.river-luatile);

    };
}
