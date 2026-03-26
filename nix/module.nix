{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.sajak-http;
in
{
  options = {
    services.sajak-http = {
      enable = lib.mkEnableOption "HTTP frontend for sajak";

      port = lib.mkOption {
        type = lib.types.port;
        default = 1983;
      };

      package = lib.mkOption {
        description = "The sajak package to use.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.sajak-http = {
      after = [ "network.target" ];
      wantedBy = [ "default.target" ];

      path = [ cfg.package ];

      script = ''
        PORT=${toString cfg.port} ${cfg.package}/bin/sajak_http
      '';
    };
  };
}
