{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [ git pkg-config openssl bashInteractive ];

  languages.rust = {
    enable = true;
  };

  processes.bot.exec = "./target/debug/algobot";
}
