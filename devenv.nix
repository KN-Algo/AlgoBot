{ pkgs, lib, config, inputs, ... }:
let
  pkgs-unstable = import inputs.nixpkgs-unstable { system = pkgs.stdenv.system; };
in
{
  packages = with pkgs; [ bashInteractive sqlx-cli openssl ];

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
  export DATABASE_URL=sqlite:bot_db.sqlite
  '';

  processes.bot.exec = "./target/debug/algo-bot";
}
