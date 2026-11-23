import atexit
import json
import os
import platform
import queue
import shutil
import subprocess
import threading
from pathlib import Path

from dotenv import dotenv_values

from roleforge import Role as BaseRole


class TailscaleError(RuntimeError):
    """Base exception for Tailscale Role failures."""


class TailscaleConfigurationError(TailscaleError):
    """Malformed or incomplete Tailscale configuration."""


class TailscaleHelperStartupError(TailscaleError):
    """The Tailscale helper process could not start."""


class TailscaleTimeoutError(TailscaleError):
    """A Tailscale operation timed out."""


class TailscaleProtocolError(TailscaleError):
    """The Tailscale helper returned invalid protocol data."""


class TailscaleHelperExitError(TailscaleError):
    """The Tailscale helper exited unexpectedly."""


_ALLOWED_SETTINGS = {"hostname", "auth_key", "state_dir"}


def _config_error(role, line, message):
    return TailscaleConfigurationError(
        "Tailscale configuration error at line {}: {}".format(line, message)
    )


def _parse_value(text, role, line):
    text = text.strip()
    if not text:
        raise _config_error(role, line, "a setting value cannot be empty")
    if text[0:1] in ('"', "'"):
        quote = text[0]
        if len(text) < 2 or text[-1] != quote:
            raise _config_error(role, line, "malformed quoted value")
        return text[1:-1]
    if " " in text:
        return text
    return text


def _parse_body(role):
    settings = {}
    env_file = None
    base = role.source.path.resolve().parent
    for offset, raw in enumerate(role.body.splitlines(), start=1):
        line = role.source.declaration_line + offset
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if "=" not in raw:
            raise _config_error(role, line, "malformed configuration line")
        key, value = (part.strip() for part in raw.split("=", 1))
        value = _parse_value(value, role, line)
        if key == "env_file":
            env_file = value
        elif key.startswith("settings."):
            setting = key[len("settings."):]
            if setting not in _ALLOWED_SETTINGS:
                raise _config_error(role, line, "unknown settings key {!r}".format(setting))
            settings[setting] = value
        else:
            raise _config_error(role, line, "unknown configuration key {!r}".format(key))

    if env_file is not None:
        env_path = Path(env_file)
        if not env_path.is_absolute():
            env_path = base / env_path
        env_path = env_path.resolve()
        try:
            values = dotenv_values(env_path)
        except Exception:
            raise TailscaleConfigurationError(
                "Tailscale configuration error: env_file could not be read: {}".format(env_path)
            ) from None
        if not env_path.is_file():
            raise TailscaleConfigurationError(
                "Tailscale configuration error: env_file could not be read: {}".format(env_path)
            )
        mapping = {
            "TAILSCALE_HOSTNAME": "hostname",
            "TAILSCALE_AUTH_KEY": "auth_key",
            "TAILSCALE_STATE_DIR": "state_dir",
        }
        for env_key, setting in mapping.items():
            if values.get(env_key) is not None:
                settings[setting] = values[env_key]

    for required in ("hostname", "auth_key", "state_dir"):
        if not settings.get(required):
            raise TailscaleConfigurationError(
                "Tailscale configuration error: required setting 'settings.{}' is missing".format(required)
            )

    state = Path(settings["state_dir"])
    if not state.is_absolute():
        state = base / state
    state = state.resolve()
    # Both IDs keep each occurrence isolated within and across source blocks.
    state = state / "roleforge-instance-{}-{}".format(role.index, role.role_index)
    settings["state_dir"] = str(state)
    return settings


class Role(BaseRole):
    """Built-in embedded Tailscale node controlled through a precompiled helper."""

    def __init__(self, role_input):
        super().__init__(role_input)
        self._process = None
        self._stdout_queue = None
        self._stderr = []
        self._lock = threading.RLock()
        self._closed = True
        self.settings = _parse_body(self)
        self.hostname = self.settings["hostname"]
        atexit.register(self._atexit_disconnect)

    def _helper_path(self):
        system = platform.system().lower()
        machine = platform.machine().lower()
        if system == "windows" and machine in ("amd64", "x86_64"):
            directory, name = "windows-amd64", "roleforge-tailscale.exe"
        elif system == "linux" and machine in ("amd64", "x86_64"):
            directory, name = "linux-amd64", "roleforge-tailscale"
        elif system == "linux" and machine in ("arm64", "aarch64"):
            directory, name = "linux-arm64", "roleforge-tailscale"
        else:
            raise TailscaleHelperStartupError(
                "No precompiled Tailscale helper is available for {} {}".format(system, machine)
            )
        helper = Path(__file__).resolve().parent / "bin" / directory / name
        if not helper.is_file():
            raise TailscaleHelperStartupError(
                "The Tailscale helper binary is missing for {} {}".format(system, machine)
            )
        return helper

    def _reader(self):
        for line in iter(self._process.stdout.readline, ""):
            if line:
                self._stdout_queue.put(line)
        self._stdout_queue.put(None)

    def _stderr_reader(self):
        for line in iter(self._process.stderr.readline, ""):
            line = line.strip()
            if line:
                self._stderr.append(line[:500])

    def _start_helper(self):
        helper = self._helper_path()
        try:
            process = subprocess.Popen(
                [str(helper)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=subprocess.PIPE, text=True, bufsize=1,
            )
        except (OSError, subprocess.SubprocessError) as exc:
            raise TailscaleHelperStartupError(
                "Could not start the Tailscale helper: {}".format(exc)
            ) from None
        self._process = process
        self._stdout_queue = queue.Queue()
        self._closed = False
        threading.Thread(target=self._reader, daemon=True).start()
        threading.Thread(target=self._stderr_reader, daemon=True).start()

    def _request(self, command, timeout=70):
        with self._lock:
            if self._process is None:
                raise TailscaleHelperExitError("The Tailscale helper is not running")
            if self._process.poll() is not None:
                self._process = None
                self._closed = True
                raise TailscaleHelperExitError("The Tailscale helper exited unexpectedly")
            try:
                self._process.stdin.write(json.dumps(command, ensure_ascii=False) + "\n")
                self._process.stdin.flush()
            except (BrokenPipeError, OSError):
                self._process = None
                self._closed = True
                raise TailscaleHelperExitError("The Tailscale helper exited unexpectedly") from None
            try:
                line = self._stdout_queue.get(timeout=timeout)
            except queue.Empty:
                raise TailscaleTimeoutError("Timed out waiting for the Tailscale helper") from None
            if line is None:
                self._process = None
                self._closed = True
                raise TailscaleHelperExitError("The Tailscale helper exited unexpectedly")
            try:
                response = json.loads(line)
            except json.JSONDecodeError:
                raise TailscaleProtocolError("The Tailscale helper returned malformed JSON") from None
            if not isinstance(response, dict) or "ok" not in response:
                raise TailscaleProtocolError("The Tailscale helper returned an invalid protocol response")
            if not response["ok"]:
                error = response.get("error")
                if not isinstance(error, str) or not error:
                    error = "The Tailscale helper returned an unknown error"
                raise TailscaleProtocolError(error)
            status = response.get("status")
            if not isinstance(status, dict):
                raise TailscaleProtocolError("The Tailscale helper response omitted status")
            return status

    def connect(self):
        with self._lock:
            if self._process is not None and self._process.poll() is None:
                return self.status()
            self._start_helper()
            try:
                return self._request({
                    "command": "connect",
                    "hostname": self.settings["hostname"],
                    "auth_key": self.settings["auth_key"],
                    "state_dir": self.settings["state_dir"],
                }, timeout=70)
            except Exception:
                self.disconnect()
                raise

    def status(self):
        with self._lock:
            if self._process is None or self._process.poll() is not None:
                return {
                    "state": "disconnected",
                    "hostname": self.hostname,
                    "tailscale_ips": [],
                    "dns_name": "",
                }
            try:
                return self._request({"command": "status"}, timeout=10)
            except (TailscaleTimeoutError, TailscaleHelperExitError, TailscaleProtocolError):
                raise

    def disconnect(self):
        with self._lock:
            if self._process is None:
                self._closed = True
                return {
                    "state": "disconnected", "hostname": self.hostname,
                    "tailscale_ips": [], "dns_name": "",
                }
            process = self._process
            try:
                self._request({"command": "disconnect"}, timeout=10)
            except Exception:
                pass
            finally:
                try:
                    if process.stdin:
                        process.stdin.close()
                except Exception:
                    pass
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                self._process = None
                self._closed = True
            return {
                "state": "disconnected", "hostname": self.hostname,
                "tailscale_ips": [], "dns_name": "",
            }

    def _atexit_disconnect(self):
        try:
            self.disconnect()
        except Exception:
            pass


def roleforge_receive(role):
    # Configuration and helper setup belong to Role.__init__; loading does not connect.
    return None
