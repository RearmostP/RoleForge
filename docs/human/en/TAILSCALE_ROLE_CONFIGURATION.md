# Tailscale Role configuration

> **Status: implemented.** The built-in Tailscale Role supports the nested `connect.settings` and `proxy.settings` syntax below. Flat `settings.hostname`, `settings.auth_key`, and `settings.state_dir` entries remain accepted for compatibility.

## `.rfg` ownership and block boundaries

RoleForge extracts an `@role` block and passes the opaque body to the Tailscale Role implementation. RoleForge does not interpret Tailscale-specific settings. The Tailscale Role owns parsing and validation.

The current RoleForge block grammar has no `@end`; a block ends at the next column-0 `@role` declaration or at EOF. Therefore the proposed example omits `@end`, which remains reserved only as ordinary body text under the current grammar.

## Proposed syntax

```text
@role Tailscale
env_file = ".env"

connect.settings {
  auth_key = ""
  hostname = "home-gallery"
  state_dir = ".roleforge/tailscale"
}

proxy.settings {
  target = "http://127.0.0.1:8000"
  listen_port = 8080
}
```

The syntax separates connection settings from proxy settings:

- `connect.settings` configures the Role-created Tailscale node.
- `proxy.settings` configures receiving requests on that node and forwarding them to a local gallery service.

Exact punctuation can remain an implementation choice, but parsing must be unambiguous and clearly documented.

## Fields

| Field | Meaning |
| --- | --- |
| `env_file` | Optional dotenv-file path. Relative paths resolve from the directory containing the `.rfg` file. |
| `connect.settings.auth_key` | Tailscale authentication key for creating or rejoining the node. The `.rfg` may contain an empty string or an example value; normally keep the real secret in `.env`. |
| `connect.settings.hostname` | Name presented for the Tailscale node created by this Role. It does not rename the operating-system computer, the Python process, or an existing system-wide Tailscale node. |
| `connect.settings.state_dir` | Persistent tsnet identity state. Retaining it lets the same node identity return after a process restart instead of relying on a fresh identity. |
| `proxy.settings.target` | Local gallery service URL to which proxy requests are forwarded. An example is `http://127.0.0.1:8000`. |
| `proxy.settings.listen_port` | Port on the Role's Tailscale node that accepts proxy requests. A client must use the matching node and port. |

Relative `state_dir` values resolve from the directory containing the source `.rfg` file, then receive a stable per-instance suffix derived from the Role's indexes. This preserves node identity across process restarts while isolating separate occurrences.

## Configuration precedence

1. Values in the `.rfg` block are valid configuration values and may serve as normal configuration or useful examples.
2. If the referenced `.env` contains the matching setting, the `.env` value overrides the `.rfg` value.
3. The `.env` file is optional; a setting absent from it retains the `.rfg` value.
4. `.env` values should use clear names. A short example, containing placeholders only, is:

```dotenv
TAILSCALE_AUTH_KEY=REPLACE_WITH_A_REAL_AUTH_KEY
TAILSCALE_HOSTNAME=home-gallery
TAILSCALE_STATE_DIR=.roleforge/tailscale
TAILSCALE_PROXY_TARGET=http://127.0.0.1:8000
TAILSCALE_PROXY_LISTEN_PORT=8080
```

The implementation currently recognizes:

- `TAILSCALE_HOSTNAME`
- `TAILSCALE_AUTH_KEY`
- `TAILSCALE_STATE_DIR`
- `TAILSCALE_PROXY_TARGET`
- `TAILSCALE_PROXY_LISTEN_PORT`

The `.env` file should be excluded from Git. Never commit or log a real authentication key.

## Missing or invalid configuration

**Implemented behavior:** loading creates the Role object without parsing or connecting. `connect()` validates connection settings and raises `TailscaleConfigurationError` for missing required values, unreadable `env_file`, malformed configuration, or unknown settings keys. `proxy.start()` validates proxy settings and raises `TailscaleConfigurationError` for malformed target or listen-port values. It raises `TailscaleConnectionError` if the node is not ready and `TailscaleProxyError` for target-reachability or listener startup failures. Exceptions identify relevant settings where possible and never expose the auth key.

Do not infer that an empty `auth_key` is valid merely because the proposed example displays it. In deployed use, authentication must normally be supplied from `.env` or the configuration source.

## Python API

```python
project = roleforge.load("project.rfg")
tailscale = project.tailscale

status = tailscale.connect()
tailscale.proxy.start()
status = tailscale.status()
tailscale.proxy.stop()
tailscale.disconnect()
```

Intended lifecycle:

- `connect()` parses and validates connection settings, then starts and connects the Role-created Tailscale node.
- `proxy.start()` requires a successfully connected node. Otherwise it returns a clear error.
- `proxy.start()` receives requests over that Tailscale node and forwards them to the configured local target.
- `proxy.stop()` stops receiving and forwarding but keeps the Tailscale node connected.
- `disconnect()` stops the proxy, if running, and then disconnects the node.
- `status()` should separate connection state from proxy state if supported by the final design.

`connect()` alone does not expose the gallery. The gallery service must be running locally, and the phone app must address the Role-created Tailscale node and the configured listening port.
