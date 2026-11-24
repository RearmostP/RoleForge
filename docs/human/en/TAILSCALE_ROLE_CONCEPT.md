# Understanding the Tailscale Role: a networking concept guide

> **Scope:** This describes the implemented gallery gateway behavior. `connect()` only connects the node; `proxy.start()` separately starts forwarding to the local gallery service.

## What Tailscale provides

Tailscale creates a private network, sometimes called a tailnet, between devices and nodes that Tailscale has authorized. Once authorized, devices can address each other by private Tailscale identities instead of exposing those services to the public internet.

The Tailscale Role can create a node through `tsnet` inside a Python program. That node is a separate Tailscale network identity, even if it runs on the same physical computer that already has a system-wide Tailscale client or another Tailscale node.

The `hostname` setting names this Role-created node. It is not a rename of the operating-system computer name. It also does not rename the Python process or change an existing system-wide Tailscale node.

## Connecting and serving are separate

Connecting the node does not automatically route all computer traffic or all internet traffic through Tailscale. It gives the embedded node a private Tailscale identity and allows authorized devices to reach services exposed by that node.

The `proxy.start()` operation is a separate step. It allows the Role's node to receive requests and forwards them to the gallery service running locally:

```text
Phone app
   |
   | HTTPS over private Tailscale network
   v
Role-created Tailscale node
   |
   | optional tsnet TLS listener (port 443 for browser access)
   v
Proxy
   |
   v
Local gallery service (for example, 127.0.0.1:8000)
```

Thus, starting `connect()` alone does not expose the gallery. The local gallery application must still be running, and the phone app must use the Role-created node and the matching listening port.

For browser clients, configure `proxy.settings.https = true` and use the node's `.ts.net` DNS name over port 443. HTTPS certificates must be enabled for the tailnet in its Tailscale admin settings. The Role's TLS listener is tailnet-only; the HTTP connection from the proxy to a loopback gallery server remains local to the same machine.

## Private gateway rather than public publishing

The proxy is a private reverse proxy. It accepts requests arriving through the Role's Tailscale node and forwards them to a local application. It does not publish the gallery to every address on the public internet.

This private gateway does not bypass Tailscale network authorization. Devices must still be authorized on the tailnet and allowed to reach the node under Tailscale's access rules.

Tailscale access and the gallery application's own login are separate security layers:

- Tailscale decides whether a device is authorized to reach the private node.
- The gallery application decides whether an authenticated user may access application data.

Both layers can be useful, but the presence of one does not replace the other.

## Persistent node identity

The Role's `state_dir` stores the tsnet node identity. If that state is retained, the node can return with the same identity after a process restart. If the state is lost or isolated differently, Tailscale may see a new node identity.

`hostname` is only the node's presented name. It does not create or rename an operating-system computer or reconfigure the existing system Tailscale client.

## Implemented lifecycle

The built-in Role implements an embedded node and connection lifecycle: `connect()`, `disconnect()`, and `status()` for the node. It also implements `proxy.start()` and `proxy.stop()` for a reverse proxy listener on the Role-created Tailscale node. The local gallery service must still be running, and live tailnet authorization and reachability require deployment verification.
