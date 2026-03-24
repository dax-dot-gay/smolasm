{ pkgs, ... }:
{
  packages = [pkgs.git pkgs.cargo-autoinherit];
  languages.rust.enable = true;
}
