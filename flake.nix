{
  description = "Keybswitch flake with auto-imported NixOS module";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    keybswitch-src = {
      url = "github:PaysanCorrezien/keybswitch";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, keybswitch-src }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        keybswitch = pkgs.rustPlatform.buildRustPackage rec {
          pname = "keybswitch";
          version = "unstable-${builtins.substring 0 8 keybswitch-src.rev}";
          src = keybswitch-src;
          cargoLock = { lockFile = "${keybswitch-src}/Cargo.lock"; };
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ systemd openssl libffi ];

          buildPhase = ''
            echo "Starting build phase"
            cargo build --release --verbose
            echo "Build phase completed"
          '';

          installPhase = ''
            echo "Starting install phase"
            mkdir -p $out/bin
            cp target/release/keybswitch $out/bin/
            echo "Install phase completed"
          '';

          meta = with pkgs.lib; {
            description = "USB Keyboard Detection and Layout Switch";
            homepage = "https://github.com/PaysanCorrezien/keybswitch";
            license = licenses.mit;
            platforms = platforms.linux;
          };
        };
      in {
        packages.default = keybswitch;
        defaultPackage = keybswitch;
        apps.default = flake-utils.lib.mkApp { drv = keybswitch; };
      }) // {
        overlays.default = final: prev: {
          keybswitch = self.packages.${prev.system}.default;
        };
        nixosModules.default = { config, lib, pkgs, ... }:
          let cfg = config.services.keybswitch;
          in {
            options.services.keybswitch = {
              enable = lib.mkEnableOption "Keybswitch service";
            };
            config = lib.mkIf cfg.enable {
              environment.systemPackages =
                [ self.packages.${pkgs.system}.default ];
              systemd.user.services.keybswitch = {
                description =
                  "Keybswitch - USB Keyboard Detection and Layout Switch";
                wantedBy = [ "default.target" ];
                after = [ "graphical-session.target" ];
                serviceConfig = {
                  ExecStart =
                    "${self.packages.${pkgs.system}.default}/bin/keybswitch";
                  Restart = "always";
                  RestartSec = "5";
                };
              };
              # Ensure the service is started for all users
              systemd.user.services.keybswitch.enable = true;
              # Allow the service to be managed without password authentication
              security.sudo.extraRules = [{
                users = [ "ALL" ];
                commands = [{
                  command =
                    "${pkgs.systemd}/bin/systemctl --user start keybswitch";
                  options = [ "NOPASSWD" ];
                }];
              }];
            };
          };
      };
}
