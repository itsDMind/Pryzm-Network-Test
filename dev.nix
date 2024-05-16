{ pkgs, ... } @ args:
let
  wasmd_release =
    "https://github.com/CosmWasm/wasmd/releases/download/v0.51.0/wasmd-v0.51.0-linux-amd64.tar.gz";
  injective_release =
    "https://github.com/InjectiveLabs/injective-chain-releases/releases/download/v1.12.1-1705909076/linux-amd64.zip";
  # "https://github.com/InjectiveLabs/injective-chain-releases/releases/download/v0.4.19-1652947015/linux-amd64.zip";

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
    injectived.exec = "DIR=$DEVENV_ROOT/.injective && LD_LIBRARY_PATH=$DIR $DIR/injectived $@ --home $DIR/injective-home";
    wasmd.exec = "$DEVENV_ROOT/.wasmd/wasmd $@";

    dev-setup-injective.exec = "$DEVENV_ROOT/.injective/injective-chain-releases/scripts/setup.sh $@";

    dev-install-wasmd.exec = ''
      DIR=$DEVENV_ROOT/.wasmd
      mkdir $DIR
      wget ${wasmd_release} -P $DIR
      tar -xzf $DIR/wasmd-v0.51.0-linux-amd64.tar.gz -C $DIR
    '';

    dev-install-cosmwasm-check.exec = "cargo install cosmwasm-check";

    dev-install-injective.exec = ''
      DIR=$DEVENV_ROOT/.injective
      mkdir $DIR
      wget ${injective_release} -P $DIR
      unzip $DIR/linux-amd64.zip -d $DIR
      git clone https://github.com/InjectiveLabs/injective-chain-releases $DIR/injective-chain-releases
    '';

    dev-remove-wasmd.exec = "rm -rf $DEVENV_ROOT/.wasmd";
    dev-remove-injective.exec = "rm -rf $DEVENV_ROOT/.injective";
  };

  enterShell = ''
  '';
}
