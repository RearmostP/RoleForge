# RoleForge v0.2.0

This GitHub Release provides installable Python Wheels for:

- Windows x64: the Wheel whose filename contains `win_amd64`
- Linux x64: the Wheel whose filename contains `linux_x86_64` or `manylinux` plus `x86_64`
- Linux ARM64: the Wheel whose filename contains `linux_aarch64` or `manylinux` plus `aarch64`

## Tailscale Role

The built-in Tailscale Role supports an independent `tsnet` node, nested `connect.settings` and `proxy.settings` configuration, `.env` overrides, persistent state, `connect()`, `proxy.start()`, `proxy.stop()`, `disconnect()`, and separate connection/proxy status reporting.

## Installation

Download the Wheel matching your OS and CPU architecture, then install it directly:

```text
python -m pip install ./roleforge-0.2.0-*.whl
```

This release is distributed through GitHub Releases, not PyPI.

## Requirements and limitations

- Python 3.10 or newer.
- Supported targets: Windows x64, Linux x64, and Linux ARM64.
- The local gallery service must be running before `proxy.start()`.
- Live Tailnet authorization, connectivity, and gallery-proxy behavior have not been end-to-end tested in this release build.
- The Tailscale node is separate from the host computer's system-wide Tailscale node and does not change the host computer name or system routing.
