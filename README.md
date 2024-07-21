# Proxy

> Some description…

## Configuration

Here is an example configuration file.

```toml
[[servers]]
listen = "127.0.0.1:25565"
hosts = [
    { hostname = "localhost", target = "127.0.0.1:25566" },
    { hostname = "localhost.localdomain", target = "127.0.0.1:25567" },
]

[[servers]]
listen = "127.0.0.1:8080"
redirect = "127.0.0.1:80"
```

## Use with Systemd

### Create the unit file

```
[Unit]
Description=Proxy
After=network.target

[Service]
WorkingDirectory=/path/to
ExecStart=/path/to/proxy
Restart=always
StandardOutput=null
StandardError=null
SyslogIdentifier=proxy

[Install]
WantedBy=multi-user.target
```

### View the logs

```shell
journalctl --follow --user-unit=proxy
```
