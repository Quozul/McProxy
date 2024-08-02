# Proxy

> A Minecraft proxy written in Rust to redirect incoming trafic from one host to another based on the host of the
> handshake packet.

---

## Installation

### Cargo

If you already have a Rust environment set up, you can use the cargo install command:

```shell
cargo install --git https://github.com/Quozul/McProxy.git
```

Cargo will build the `proxy` binary and place it in `$HOME/.cargo`.

---

## Configuration

The program searches for a configuration file named `config.toml` in the same directory as the binary itself. The
configuration file is required by the program.

### Overriding the Configuration File

If you need to use a different configuration file or specify a different path, you can override the default behavior by
using the `--config` argument when running the program. This allows you to point to any configuration file you prefer.

### Configuration File Format

The configuration file must be in TOML format. Here is an example of what the `config.toml` might look like:

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

---

## Running using a Systemd service

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
