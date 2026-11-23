package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"sync"
	"time"

	"tailscale.com/tsnet"
)

type command struct {
	Command  string `json:"command"`
	Hostname string `json:"hostname,omitempty"`
	AuthKey  string `json:"auth_key,omitempty"`
	StateDir string `json:"state_dir,omitempty"`
}

type response struct {
	Ok bool `json:"ok"`
	Status map[string]any `json:"status,omitempty"`
	Error string `json:"error,omitempty"`
}

func writeResponse(w *json.Encoder, r response) {
	if err := w.Encode(r); err != nil {
		fmt.Fprintln(os.Stderr, "protocol write failed")
	}
}

func statusMap(srv *tsnet.Server, hostname string) (map[string]any, error) {
	if srv == nil {
		return map[string]any{"state":"disconnected","hostname":hostname,"tailscale_ips":[]string{},"dns_name":""}, nil
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
	for _, ip := range st.TailscaleIPs { ips = append(ips, ip.String()) }
	dns := ""
	if st.Self != nil { dns = st.Self.DNSName }
	return map[string]any{
		"state": st.BackendState,
		"hostname": hostname,
		"tailscale_ips": ips,
		"dns_name": dns,
	}, nil
}

func main() {
	scan := bufio.NewScanner(os.Stdin)
	scan.Buffer(make([]byte, 4096), 1024*1024)
	enc := json.NewEncoder(os.Stdout)
	var mu sync.Mutex
	var srv *tsnet.Server
	var hostname string

	for scan.Scan() {
		var cmd command
		if err := json.Unmarshal(scan.Bytes(), &cmd); err != nil {
			writeResponse(enc, response{Ok:false, Error:"malformed JSON command"})
			continue
		}
		mu.Lock()
		switch cmd.Command {
		case "connect":
			if cmd.Hostname == "" || cmd.AuthKey == "" || cmd.StateDir == "" {
				writeResponse(enc, response{Ok:false, Error:"connect requires hostname, auth_key, and state_dir"})
				mu.Unlock(); continue
			}
			if srv != nil { status, err := statusMap(srv, hostname); if err == nil { writeResponse(enc, response{Ok:true, Status:status}); mu.Unlock(); continue } }
			hostname = cmd.Hostname
			srv = &tsnet.Server{Hostname: cmd.Hostname, AuthKey: cmd.AuthKey, Dir: cmd.StateDir}
			ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
			st, err := srv.Up(ctx)
			cancel()
			if err != nil {
				writeResponse(enc, response{Ok:false, Error:"Tailscale connection failed"})
				_ = srv.Close(); srv = nil
				mu.Unlock(); continue
			}
			ips := make([]string, 0, len(st.TailscaleIPs))
			for _, ip := range st.TailscaleIPs { ips = append(ips, ip.String()) }
			dns := ""; if st.Self != nil { dns = st.Self.DNSName }
			writeResponse(enc, response{Ok:true, Status:map[string]any{"state":st.BackendState,"hostname":hostname,"tailscale_ips":ips,"dns_name":dns}})
		case "status":
			status, err := statusMap(srv, hostname)
			if err != nil { writeResponse(enc, response{Ok:false, Error:"unable to read Tailscale status"}) } else { writeResponse(enc, response{Ok:true, Status:status}) }
		case "disconnect":
			if srv != nil { if err := srv.Close(); err != nil { writeResponse(enc, response{Ok:false, Error:"Tailscale disconnect failed"}); mu.Unlock(); continue }; srv = nil }
			hostname = ""
			writeResponse(enc, response{Ok:true, Status:map[string]any{"state":"disconnected","hostname":"","tailscale_ips":[]string{},"dns_name":""}})
		default:
			writeResponse(enc, response{Ok:false, Error:"unknown command"})
		}
		mu.Unlock()
	}
	mu.Lock()
	if srv != nil { _ = srv.Close() }
	mu.Unlock()
}
