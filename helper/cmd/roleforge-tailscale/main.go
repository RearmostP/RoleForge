package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"sync"
	"time"

	"tailscale.com/tsnet"
)

type command struct {
	Command    string `json:"command"`
	Hostname   string `json:"hostname,omitempty"`
	AuthKey    string `json:"auth_key,omitempty"`
	StateDir   string `json:"state_dir,omitempty"`
	Target     string `json:"target,omitempty"`
	ListenPort int    `json:"listen_port,omitempty"`
	HTTPS      bool   `json:"https,omitempty"`
}

type response struct {
	Ok        bool           `json:"ok"`
	Status    map[string]any `json:"status,omitempty"`
	Error     string         `json:"error,omitempty"`
	ErrorCode string         `json:"error_code,omitempty"`
}

func writeResponse(w *json.Encoder, r response) {
	if err := w.Encode(r); err != nil {
		fmt.Fprintln(os.Stderr, "protocol write failed")
	}
}

func ensureStateDir(path string) error {
	return os.MkdirAll(path, 0700)
}

func statusMap(srv *tsnet.Server, hostname, proxyState string, listenPort int, proxyHTTPS bool) (map[string]any, error) {
	if srv == nil {
		return map[string]any{"state": "disconnected", "hostname": hostname, "tailscale_ips": []string{}, "dns_name": "", "proxy_state": "stopped", "listen_port": listenPort, "proxy_https": false}, nil
	}
	lc, err := srv.LocalClient()
	if err != nil {
		return nil, err
	}
	st, err := lc.Status(context.Background())
	if err != nil {
		return nil, err
	}
	ips := make([]string, 0, len(st.TailscaleIPs))
	for _, ip := range st.TailscaleIPs {
		ips = append(ips, ip.String())
	}
	dns := ""
	if st.Self != nil {
		dns = st.Self.DNSName
	}
	return map[string]any{"state": st.BackendState, "hostname": hostname, "tailscale_ips": ips, "dns_name": dns, "proxy_state": proxyState, "listen_port": listenPort, "proxy_https": proxyHTTPS}, nil
}

func nodeReady(srv *tsnet.Server) bool {
	if srv == nil {
		return false
	}
	lc, err := srv.LocalClient()
	if err != nil {
		return false
	}
	st, err := lc.Status(context.Background())
	return err == nil && st.BackendState == "Running"
}

func stopProxy(listener *net.Listener, server **http.Server) {
	if *server != nil {
		_ = (*server).Close()
		*server = nil
	}
	if listener != nil && *listener != nil {
		_ = (*listener).Close()
		*listener = nil
	}
}

func main() {
	scan := bufio.NewScanner(os.Stdin)
	scan.Buffer(make([]byte, 4096), 1024*1024)
	enc := json.NewEncoder(os.Stdout)
	var mu sync.Mutex
	var srv *tsnet.Server
	var hostname string
	var proxyState = "stopped"
	var listenPort int
	var proxyHTTPS bool
	var proxyServer *http.Server
	var proxyListener net.Listener

	for scan.Scan() {
		var cmd command
		if err := json.Unmarshal(scan.Bytes(), &cmd); err != nil {
			writeResponse(enc, response{Ok: false, Error: "malformed JSON command", ErrorCode: "malformed_protocol"})
			continue
		}
		mu.Lock()
		switch cmd.Command {
		case "connect":
			if cmd.Hostname == "" || cmd.AuthKey == "" || cmd.StateDir == "" {
				writeResponse(enc, response{Ok: false, Error: "connect requires hostname, auth_key, and state_dir", ErrorCode: "invalid_config"})
				mu.Unlock()
				continue
			}
			if srv != nil {
				if nodeReady(srv) {
					status, err := statusMap(srv, hostname, proxyState, listenPort, proxyHTTPS)
					if err == nil {
						writeResponse(enc, response{Ok: true, Status: status})
						mu.Unlock()
						continue
					}
				}
			}
			hostname = cmd.Hostname
			proxyHTTPS = false
			if err := ensureStateDir(cmd.StateDir); err != nil {
				writeResponse(enc, response{Ok: false, Error: "could not prepare Tailscale state directory", ErrorCode: "connection_failed"})
				mu.Unlock()
				continue
			}
			srv = &tsnet.Server{Hostname: cmd.Hostname, AuthKey: cmd.AuthKey, Dir: cmd.StateDir}
			ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
			st, err := srv.Up(ctx)
			cancel()
			if err != nil {
				writeResponse(enc, response{Ok: false, Error: "Tailscale connection failed", ErrorCode: "connection_failed"})
				_ = srv.Close()
				srv = nil
				mu.Unlock()
				continue
			}
			if st.BackendState != "Running" {
				writeResponse(enc, response{Ok: false, Error: "Tailscale node was not ready", ErrorCode: "connection_failed"})
				_ = srv.Close()
				srv = nil
				mu.Unlock()
				continue
			}
			ips := make([]string, 0, len(st.TailscaleIPs))
			for _, ip := range st.TailscaleIPs {
				ips = append(ips, ip.String())
			}
			dns := ""
			if st.Self != nil {
				dns = st.Self.DNSName
			}
			proxyState = "stopped"
			writeResponse(enc, response{Ok: true, Status: map[string]any{"state": st.BackendState, "hostname": hostname, "tailscale_ips": ips, "dns_name": dns, "proxy_state": proxyState, "listen_port": listenPort, "proxy_https": proxyHTTPS}})
		case "status":
			status, err := statusMap(srv, hostname, proxyState, listenPort, proxyHTTPS)
			if err != nil {
				writeResponse(enc, response{Ok: false, Error: "unable to read Tailscale status", ErrorCode: "protocol_error"})
			} else {
				writeResponse(enc, response{Ok: true, Status: status})
			}
		case "proxy_start":
			if !nodeReady(srv) {
				writeResponse(enc, response{Ok: false, Error: "Tailscale node is not connected and ready", ErrorCode: "node_not_ready"})
				mu.Unlock()
				continue
			}
			if proxyState == "running" {
				status, _ := statusMap(srv, hostname, proxyState, listenPort, proxyHTTPS)
				writeResponse(enc, response{Ok: true, Status: status})
				mu.Unlock()
				continue
			}
			if cmd.ListenPort < 1 || cmd.ListenPort > 65535 {
				writeResponse(enc, response{Ok: false, Error: "proxy listen_port must be between 1 and 65535", ErrorCode: "invalid_proxy_config"})
				mu.Unlock()
				continue
			}
			target, err := url.Parse(cmd.Target)
			if err != nil || target.Host == "" || (target.Scheme != "http" && target.Scheme != "https") {
				writeResponse(enc, response{Ok: false, Error: "proxy target must be a valid HTTP or HTTPS URL", ErrorCode: "invalid_proxy_config"})
				mu.Unlock()
				continue
			}
			probeHost := target.Host
			if target.Port() == "" {
				if target.Scheme == "http" {
					probeHost = net.JoinHostPort(target.Hostname(), "80")
				} else {
					probeHost = net.JoinHostPort(target.Hostname(), "443")
				}
			}
			probe, err := net.DialTimeout("tcp", probeHost, 3*time.Second)
			if err != nil {
				writeResponse(enc, response{Ok: false, Error: "configured local proxy target is unreachable", ErrorCode: "proxy_target_unreachable"})
				mu.Unlock()
				continue
			}
			probe.Close()
			var listener net.Listener
			if cmd.HTTPS {
				listener, err = srv.ListenTLS("tcp", fmt.Sprintf(":%d", cmd.ListenPort))
			} else {
				listener, err = srv.Listen("tcp", fmt.Sprintf(":%d", cmd.ListenPort))
			}
			if err == nil {
				addr := listener.Addr().String()
				_, port, _ := net.SplitHostPort(addr)
				if port != "" {
					fmt.Sscan(port, &listenPort)
				}
			}
			if err != nil {
				writeResponse(enc, response{Ok: false, Error: "could not start the tailnet proxy listener", ErrorCode: "proxy_start_failed"})
				mu.Unlock()
				continue
			}
			proxyListener = listener
			proxy := httputil.NewSingleHostReverseProxy(target)
			proxyServer = &http.Server{Handler: proxy, ReadHeaderTimeout: 5 * time.Second}
			listenPort = cmd.ListenPort
			proxyHTTPS = cmd.HTTPS
			proxyState = "running"
			go func(s *http.Server, ln net.Listener) { _ = s.Serve(ln) }(proxyServer, proxyListener)
			status, _ := statusMap(srv, hostname, proxyState, listenPort, proxyHTTPS)
			writeResponse(enc, response{Ok: true, Status: status})
		case "proxy_stop":
			if proxyServer != nil || proxyListener != nil {
				stopProxy(&proxyListener, &proxyServer)
			}
			proxyState = "stopped"
			proxyHTTPS = false
			status, _ := statusMap(srv, hostname, proxyState, listenPort, proxyHTTPS)
			writeResponse(enc, response{Ok: true, Status: status})
		case "disconnect":
			stopProxy(&proxyListener, &proxyServer)
			proxyState = "stopped"
			proxyHTTPS = false
			if srv != nil {
				if err := srv.Close(); err != nil {
					writeResponse(enc, response{Ok: false, Error: "Tailscale disconnect failed", ErrorCode: "disconnect_failed"})
					mu.Unlock()
					continue
				}
			}
			srv = nil
			hostname = ""
			listenPort = 0
			writeResponse(enc, response{Ok: true, Status: map[string]any{"state": "disconnected", "hostname": "", "tailscale_ips": []string{}, "dns_name": "", "proxy_state": "stopped", "listen_port": 0, "proxy_https": false}})
		default:
			writeResponse(enc, response{Ok: false, Error: "unknown command", ErrorCode: "unknown_command"})
		}
		mu.Unlock()
	}
	mu.Lock()
	stopProxy(&proxyListener, &proxyServer)
	if srv != nil {
		_ = srv.Close()
	}
	mu.Unlock()
}
