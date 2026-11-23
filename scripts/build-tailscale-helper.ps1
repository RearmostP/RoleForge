$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$helperRoot = Join-Path $root 'helper'
$outRoot = Join-Path $root 'roleforge/python/roleforge/builtin_roles/Tailscale/bin'

if (-not (Get-Command go -ErrorAction SilentlyContinue)) {
    throw 'Go toolchain is required. Missing: go executable'
}

$targets = @(
    @{ GOOS='windows'; GOARCH='amd64'; Dir='windows-amd64'; Name='roleforge-tailscale.exe' },
    @{ GOOS='linux'; GOARCH='amd64'; Dir='linux-amd64'; Name='roleforge-tailscale' },
    @{ GOOS='linux'; GOARCH='arm64'; Dir='linux-arm64'; Name='roleforge-tailscale' }
)

Push-Location $helperRoot
try {
    go mod download
    foreach ($target in $targets) {
        $outDir = Join-Path $outRoot $target.Dir
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
        $env:GOOS = $target.GOOS
        $env:GOARCH = $target.GOARCH
        $env:CGO_ENABLED = '0'
        go build -trimpath -ldflags '-s -w' -o (Join-Path $outDir $target.Name) ./cmd/roleforge-tailscale
        if ($LASTEXITCODE -ne 0) { throw "Go build failed for $($target.GOOS)/$($target.GOARCH)" }
    }
}
finally {
    Remove-Item Env:GOOS -ErrorAction SilentlyContinue
    Remove-Item Env:GOARCH -ErrorAction SilentlyContinue
    Remove-Item Env:CGO_ENABLED -ErrorAction SilentlyContinue
    Pop-Location
}
