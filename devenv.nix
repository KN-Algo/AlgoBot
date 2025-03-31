{ pkgs, lib, config, inputs, ... }:
let
  pkgs-unstable = import inputs.nixpkgs-unstable { system = pkgs.stdenv.system; };
in
{
  packages = with pkgs; [ bashInteractive ];

  languages = {
    rust = {
      enable = true;
      toolchain = {
        cargo = pkgs-unstable.cargo;
        rustc = pkgs-unstable.rustc;
        clippy = pkgs-unstable.clippy;
        rust-analyzer = pkgs-unstable.rust-analyzer;
        rustfmt = pkgs-unstable.rustfmt;
      };
    };
  };

  enterShell = ''
  export DISCORD_TOKEN=$(cat token)
  '';

  processes.bot.exec = "./target/debug/algo-bot";
}
