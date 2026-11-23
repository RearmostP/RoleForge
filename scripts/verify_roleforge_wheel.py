from pathlib import Path
from zipfile import ZipFile
import sys

TARGET_TO_TAG = {
    "x86_64-pc-windows-msvc": ("win", "amd64"),
    "x86_64-unknown-linux-gnu": ("linux", "x86_64"),
    "aarch64-unknown-linux-gnu": ("linux", "aarch64"),
}

if len(sys.argv) != 2:
    raise SystemExit("usage: verify_roleforge_wheel.py RUST_TARGET")

target = sys.argv[1]
platform_tag, arch_tag = TARGET_TO_TAG[target]
wheels = list(Path("target/wheels").glob("*.whl"))
if not wheels:
    raise SystemExit("No Wheel was built")
wheel = max(wheels, key=lambda path: path.stat().st_mtime)
name = wheel.name.lower()
if platform_tag not in name:
    raise SystemExit(f"Wheel filename does not match {platform_tag}: {wheel.name}")
if arch_tag not in name:
    raise SystemExit(f"Wheel filename does not match {arch_tag}: {wheel.name}")

required = (
    "roleforge/builtin_roles/Tailscale/main.py",
    "roleforge/builtin_roles/Tailscale/bin/windows-amd64/roleforge-tailscale.exe",
    "roleforge/builtin_roles/Tailscale/bin/linux-amd64/roleforge-tailscale",
    "roleforge/builtin_roles/Tailscale/bin/linux-arm64/roleforge-tailscale",
)
with ZipFile(wheel) as archive:
    names = set(archive.namelist())
    missing = [path for path in required if path not in names]
    extensions = [path for path in names if path.startswith("roleforge/_native") and path.endswith(".so")]
    if platform_tag == "win":
        extensions = [path for path in names if path.startswith("roleforge/_native") and path.endswith(".pyd")]
    if missing:
        raise SystemExit("Missing required Wheel paths: " + ", ".join(missing))
    if not extensions:
        raise SystemExit("Missing the platform Rust extension")
print(f"Verified {wheel.name}: {target}; native extension={extensions[0]}; helpers=3")
