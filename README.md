# Proxy

> A proxy written in Rust to redirect incoming trafic from one host to another.

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

## Installation using Systemd

1. Create a new unit file: `/etc/systemd/system/proxy.service`
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

   [Install]
   WantedBy=multi-user.target
   ```
2. Reload the daemon:
   ```shell
   sudo systemctl daemon-reload
   ```
3. Enable and start the service:
   ```shell
   sudo systemctl enable --now proxy
   ```
4. Read the logs:
   ```shell
   # View all logs and follow for new ones
   journalctl --follow --unit=proxy
   # View only error logs
   journalctl --unit=proxy --priority=4 --pager-end
   ```
