package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestEnsureStateDirCreatesPersistentPath(t *testing.T) {
	path := filepath.Join(t.TempDir(), "state", "tailscale")
	if err := ensureStateDir(path); err != nil {
		t.Fatalf("ensureStateDir() error = %v", err)
	}
	if err := ensureStateDir(path); err != nil {
		t.Fatalf("ensureStateDir() on existing path error = %v", err)
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("state directory was not created: %v", err)
	}
	if !info.IsDir() {
		t.Fatal("state path is not a directory")
	}
}
