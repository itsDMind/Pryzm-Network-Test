{ pkgs, ... } @ args:
let
  wasmd_release =
    "https://github.com/CosmWasm/wasmd/releases/download/v0.51.0/wasmd-v0.51.0-linux-amd64.tar.gz";
  injective_release =
    "https://github.com/InjectiveLabs/injective-chain-releases/releases/download/v0.4.19-1652947015/linux-amd64.zip";
in
{
  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [
      "wasm32-unknown-unknown"
    ];
  };

  # https://devenv.sh/reference/options/
  packages = [
    # # package example
    # pkgs.hello
  ];

  # # processes example
  # processes.hello.exec = "hello";

  scripts = {
    injectived.exec = "DIR=$DEVENV_ROOT/.injective && LD_LIBRARY_PATH=$DIR $DIR/injectived $@";
    injective-setup.exec = "$DEVENV_ROOT/.injective/injective-chain-releases/scripts/setup.sh $@";

    wasmd.exec = "$DEVENV_ROOT/.wasmd/wasmd $@";

    install-wasmd.exec = ''
      DIR=$DEVENV_ROOT/.wasmd
      wget ${wasmd_release} -P $DIR
      tar -xzf $DIR/wasmd-v0.51.0-linux-amd64.tar.gz -C $DIR
    '';

    install-cosmwasm-check.exec = "cargo install cosmwasm-check";

    install-injective.exec = ''
      DIR=$DEVENV_ROOT/.injective
      wget ${injective_release} -P $DIR
      unzip $DIR/linux-amd64.zip -d $DIR
      git clone https://github.com/InjectiveLabs/injective-chain-releases $DIR/injective-chain-releases
    '';
  };

  enterShell = ''
  '';
}
