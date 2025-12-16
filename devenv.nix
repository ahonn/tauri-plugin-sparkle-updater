{ pkgs, lib, config, inputs, ... }:
{
  dotenv.enable = true;

  packages = [
    pkgs.rustup
  ];

  languages.javascript = {
    enable = true;
    package = pkgs.nodejs;
    corepack.enable = true;
  };

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  enterShell = ''
    rustupHomeDir="$DEVENV_ROOT/.rustup"
    mkdir -p "$rustupHomeDir"
    export RUSTUP_HOME="$rustupHomeDir"
    rustup default stable 2>/dev/null || true
  '';
}
