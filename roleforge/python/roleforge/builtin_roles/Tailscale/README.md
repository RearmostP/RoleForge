# Built-in Tailscale Role

`Tailscale` is a built-in Python Role that manages one embedded `tsnet` node. It does not turn on or off a computer or a system-wide Tailscale VPN and does not require separately installed `tailscaled` or the Tailscale CLI.

## `.rfg` configuration

```text
@role Tailscale
env_file = "./.env"
settings.hostname = "example-device"
settings.auth_key = "REPLACE_WITH_A_REAL_AUTH_KEY"
settings.state_dir = ".roleforge/tailscale"
```

`env_file` is optional. Relative `env_file` and `settings.state_dir` paths are resolved from `role.source.path.parent`. An `.env` file is read with `python-dotenv` without changing the process environment.

| `.env` key | `.rfg` setting |
| --- | --- |
| `TAILSCALE_HOSTNAME` | `settings.hostname` |
| `TAILSCALE_AUTH_KEY` | `settings.auth_key` |
| `TAILSCALE_STATE_DIR` | `settings.state_dir` |

Matching `.env` values override direct `.rfg` values. Unrelated environment variables are ignored. `hostname`, `auth_key`, and `state_dir` are required after merging.

If `env_file` is specified but missing or unreadable, loading raises `TailscaleConfigurationError`. Unknown or malformed settings raise a configuration error identifying the Tailscale Role and line/key. Real auth keys belong only in an ignored `.env` file; never commit or log them.

## Python API

Loading a `.rfg` file creates and configures the Role only. It does not connect:

```python
project = roleforge.load("project.rfg")

project.tailscale.status()
project.tailscale.connect()
project.tailscale.disconnect()
```

- `connect()` starts the embedded helper and waits for Tailscale readiness.
- `disconnect()` closes the helper and is safe when already disconnected.
- `status()` reports existing state without starting a connection.

Repeated `connect()` calls are safe. A later `connect()` starts a fresh helper after `disconnect()`.

`status()` returns a dictionary with:

```python
{
    "state": "Running",
    "hostname": "example-device",
    "tailscale_ips": ["100.x.y.z"],
    "dns_name": "example-device.tailnet.ts.net",
}
```

For a disconnected Role, `tailscale_ips` is empty and `dns_name` is an empty string.

## State isolation and supported binaries

Relative state directories are isolated per Tailscale Role occurrence using RoleForge's global and per-name Role indexes.

Precompiled helpers are stored under `bin/`:

- `windows-amd64/roleforge-tailscale.exe`
- `linux-amd64/roleforge-tailscale`
- `linux-arm64/roleforge-tailscale`

The Python Role selects the helper by the current operating system and CPU architecture.
