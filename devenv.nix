{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  languages.rust.enable = true;

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };
}
