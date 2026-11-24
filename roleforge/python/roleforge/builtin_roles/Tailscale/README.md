# Built-in Tailscale Role

`Tailscale` is a built-in Python Role that manages one embedded `tsnet` node. It does not turn on or off a computer or a system-wide Tailscale VPN and does not require separately installed `tailscaled` or the Tailscale CLI.

## `.rfg` configuration

```text
@role Tailscale
env_file = "./.env"

connect.settings {
  hostname = "home-gallery"
  auth_key = ""
  state_dir = ".roleforge/tailscale"
}

proxy.settings {
  https = true
  target = "http://127.0.0.1:8000"
  listen_port = 443
}
```

With `https = true`, the Role uses `tsnet.Server.ListenTLS` for an HTTPS listener available only on the tailnet. Enable HTTPS certificates for the tailnet in its Tailscale admin settings, and use the node's `.ts.net` DNS name over port 443. This keeps browser cryptography in a secure context. `https` defaults to `false` for compatibility with existing configurations; do not use that mode for a browser gallery that relies on secure-context APIs.

`env_file` is optional. Relative `env_file` and `settings.state_dir` paths are resolved from `role.source.path.parent`. An `.env` file is read with `python-dotenv` without changing the process environment.

| `.env` key | `.rfg` setting |
| --- | --- |
| `TAILSCALE_HOSTNAME` | `connect.settings.hostname` |
| `TAILSCALE_AUTH_KEY` | `connect.settings.auth_key` |
| `TAILSCALE_STATE_DIR` | `connect.settings.state_dir` |

Matching `.env` values override direct `.rfg` values. Unrelated environment variables are ignored. `hostname`, `auth_key`, and `state_dir` are required after merging. Proxy settings are `target`, `listen_port`, and optional `https` (default `false`); their `.env` names are `TAILSCALE_PROXY_TARGET`, `TAILSCALE_PROXY_LISTEN_PORT`, and `TAILSCALE_PROXY_HTTPS`.

If `env_file` is specified but missing or unreadable, calling `connect()` raises `TailscaleConfigurationError`. Unknown or malformed settings also raise a configuration error identifying the Tailscale Role and line/key. Real auth keys belong only in an ignored `.env` file; never commit or log them.

## Python API

Loading a `.rfg` file creates the Role object only. Configuration is parsed and validated by the first `connect()` call. Loading does not read the `.env` file or connect:

```python
project = roleforge.load("project.rfg")

project.tailscale.connect()
project.tailscale.proxy.start()
project.tailscale.status()
project.tailscale.proxy.stop()
project.tailscale.disconnect()
```

- `connect()` starts the embedded helper and waits for Tailscale readiness; it does not start the proxy.
- `proxy.start()` requires the node to be ready and forwards tailnet requests to the configured local target, using HTTPS on the tailnet when `https = true`.
- `proxy.stop()` stops forwarding but leaves the node connected.
- `disconnect()` stops the proxy if running and closes the helper; it is safe when already disconnected.
- `status()` reports connection and proxy state separately without starting a connection.

Repeated `connect()` calls are safe. A later `connect()` starts a fresh helper after `disconnect()`.

`status()` returns a dictionary with:

```python
{
    "state": "Running",
    "hostname": "example-device",
    "tailscale_ips": ["100.x.y.z"],
    "dns_name": "example-device.tailnet.ts.net",
    "proxy_state": "running",
    "listen_port": 443,
    "proxy_https": true,
}
```

`proxy_https` reports whether the active listener uses TLS. For a disconnected Role, `tailscale_ips` is empty, `dns_name` is an empty string, `proxy_state` is `stopped`, `listen_port` is `0`, and `proxy_https` is `false`.

## State isolation and supported binaries

Relative state directories are isolated per Tailscale Role occurrence using RoleForge's global and per-name Role indexes.

Precompiled helpers are stored under `bin/`:

- `windows-amd64/roleforge-tailscale.exe`
- `linux-amd64/roleforge-tailscale`
- `linux-arm64/roleforge-tailscale`

The Python Role selects the helper by the current operating system and CPU architecture.
