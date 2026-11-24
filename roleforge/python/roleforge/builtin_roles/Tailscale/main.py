import atexit
import json
import platform
import queue
import subprocess
import threading
from pathlib import Path
from urllib.parse import urlparse

from dotenv import dotenv_values

from roleforge import Role as BaseRole


class TailscaleError(RuntimeError):
    """Base exception for Tailscale Role failures."""


class TailscaleConfigurationError(TailscaleError):
    """Malformed or incomplete Tailscale configuration."""


class TailscaleConnectionError(TailscaleError):
    """The Tailscale node could not connect or become ready."""


class TailscaleProxyError(TailscaleError):
    """The tailnet proxy could not start or stop safely."""


class TailscaleHelperStartupError(TailscaleError):
    """The Tailscale helper process could not start."""


class TailscaleTimeoutError(TailscaleError):
    """A Tailscale operation timed out."""


class TailscaleProtocolError(TailscaleError):
    """The Tailscale helper returned invalid protocol data."""


class TailscaleHelperExitError(TailscaleError):
    """The Tailscale helper exited unexpectedly."""


_ALLOWED_CONNECT_SETTINGS = {"hostname", "auth_key", "state_dir"}
_ALLOWED_PROXY_SETTINGS = {"target", "listen_port", "https"}


def _config_error(role, line, message):
    return TailscaleConfigurationError(
        "Tailscale configuration error at line {}: {}".format(line, message)
    )


def _parse_value(text, role, line):
    text = text.strip()
    if text[0:1] in ('"', "'"):
        quote = text[0]
        if len(text) < 2 or text[-1] != quote:
            raise _config_error(role, line, "malformed quoted value")
        return text[1:-1]
    if not text:
        raise _config_error(role, line, "a setting value cannot be empty")
    if " " in text:
        return text
    return text


def _parse_body(role):
    connect = {}
    proxy = {}
    env_file = None
    base = role.source.path.resolve().parent
    section = None

    for offset, raw in enumerate(role.body.splitlines(), start=1):
        line = role.source.declaration_line + offset
        stripped = raw.strip()
        if not stripped or stripped.startswith("#") or stripped == "@end":
            continue
        if stripped == "connect.settings {":
            section = "connect"
            continue
        if stripped == "proxy.settings {":
            section = "proxy"
            continue
        if stripped == "}":
            section = None
            continue
        if "=" not in raw:
            raise _config_error(role, line, "malformed configuration line")
        key, value = (part.strip() for part in raw.split("=", 1))
        value = _parse_value(value, role, line)
        if section is None:
            if key == "env_file":
                env_file = value
            elif key.startswith("settings."):
                setting = key[len("settings."):]
                if setting not in _ALLOWED_CONNECT_SETTINGS:
                    raise _config_error(role, line, "unknown settings key {!r}".format(setting))
                connect[setting] = value
            else:
                raise _config_error(role, line, "unknown configuration key {!r}".format(key))
        elif section == "connect":
            if key not in _ALLOWED_CONNECT_SETTINGS:
                raise _config_error(role, line, "unknown connect.settings key {!r}".format(key))
            connect[key] = value
        else:
            if key not in _ALLOWED_PROXY_SETTINGS:
                raise _config_error(role, line, "unknown proxy.settings key {!r}".format(key))
            proxy[key] = value

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
            "TAILSCALE_HOSTNAME": ("connect", "hostname"),
            "TAILSCALE_AUTH_KEY": ("connect", "auth_key"),
            "TAILSCALE_STATE_DIR": ("connect", "state_dir"),
            "TAILSCALE_PROXY_TARGET": ("proxy", "target"),
            "TAILSCALE_PROXY_LISTEN_PORT": ("proxy", "listen_port"),
            "TAILSCALE_PROXY_HTTPS": ("proxy", "https"),
        }
        for env_key, (section_name, setting) in mapping.items():
            if values.get(env_key) is not None:
                destination = connect if section_name == "connect" else proxy
                destination[setting] = values[env_key]

    for required in ("hostname", "auth_key", "state_dir"):
        if not connect.get(required):
            raise TailscaleConfigurationError(
                "Tailscale configuration error: required setting 'connect.settings.{}' is missing".format(required)
            )

    state = Path(connect["state_dir"])
    if not state.is_absolute():
        state = base / state
    state = state.resolve()
    state = state / "roleforge-instance-{}-{}".format(role.index, role.role_index)
    connect["state_dir"] = str(state)

    https = proxy.get("https", "false").lower()
    if https not in ("true", "false"):
        raise TailscaleConfigurationError(
            "Tailscale configuration error: 'proxy.settings.https' must be true or false"
        )
    proxy["https"] = https == "true"

    return {"connect": connect, "proxy": proxy}


class _TailscaleProxy:
    def __init__(self, role):
        self._role = role

    def start(self):
        with self._role._lock:
            if self._role._process is None or self._role._process.poll() is not None:
                raise TailscaleConnectionError("Tailscale node is not connected and ready")
            if self._role.settings is None:
                self._role.settings = _parse_body(self._role)
                self._role.connect_settings = self._role.settings["connect"]
                self._role.proxy_settings = self._role.settings["proxy"]
                self._role.hostname = self._role.connect_settings["hostname"]
            proxy = self._role.proxy_settings
            target = proxy.get("target")
            if not target:
                raise TailscaleConfigurationError("Tailscale configuration error: required setting 'proxy.settings.target' is missing")
            parsed = urlparse(target)
            if parsed.scheme not in ("http", "https") or not parsed.hostname:
                raise TailscaleConfigurationError("Tailscale configuration error: 'proxy.settings.target' must be a valid HTTP or HTTPS URL")
            try:
                listen_port = int(proxy.get("listen_port", ""))
            except (TypeError, ValueError):
                raise TailscaleConfigurationError("Tailscale configuration error: 'proxy.settings.listen_port' must be an integer") from None
            if listen_port < 1 or listen_port > 65535:
                raise TailscaleConfigurationError("Tailscale configuration error: 'proxy.settings.listen_port' must be between 1 and 65535")
            return self._role._request({
                "command": "proxy_start",
                "target": target,
                "listen_port": listen_port,
                "https": proxy["https"],
            }, timeout=15)

    def stop(self):
        with self._role._lock:
            if self._role._process is None or self._role._process.poll() is not None:
                return {
                    "state": "disconnected", "hostname": self._role.hostname or "",
                    "tailscale_ips": [], "dns_name": "",
                    "proxy_state": "stopped", "listen_port": 0, "proxy_https": False,
                }
            return self._role._request({"command": "proxy_stop"}, timeout=10)


class Role(BaseRole):
    """Built-in embedded Tailscale node controlled through a precompiled helper."""

    def __init__(self, role_input):
        super().__init__(role_input)
        self._process = None
        self._stdout_queue = None
        self._stderr = []
        self._lock = threading.RLock()
        self._closed = True
        self.settings = None
        self.connect_settings = None
        self.proxy_settings = None
        self.hostname = None
        self._proxy = _TailscaleProxy(self)
        atexit.register(self._atexit_disconnect)

    @property
    def proxy(self):
        return self._proxy

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
                code = response.get("error_code")
                if code == "invalid_config":
                    raise TailscaleConfigurationError(error)
                if code == "connection_failed" or code == "node_not_ready":
                    raise TailscaleConnectionError(error)
                if code in ("invalid_proxy_config", "proxy_target_unreachable", "proxy_start_failed"):
                    raise TailscaleProxyError(error)
                raise TailscaleProtocolError(error)
            status = response.get("status")
            if not isinstance(status, dict):
                raise TailscaleProtocolError("The Tailscale helper response omitted status")
            return status

    def connect(self):
        with self._lock:
            if self.settings is None:
                self.settings = _parse_body(self)
                self.connect_settings = self.settings["connect"]
                self.proxy_settings = self.settings["proxy"]
                self.hostname = self.connect_settings["hostname"]
            if self._process is not None and self._process.poll() is None:
                current = self.status()
                if current.get("state") == "Running":
                    return current
            self._start_helper()
            try:
                return self._request({
                    "command": "connect",
                    "hostname": self.connect_settings["hostname"],
                    "auth_key": self.connect_settings["auth_key"],
                    "state_dir": self.connect_settings["state_dir"],
                }, timeout=70)
            except Exception:
                self.disconnect()
                raise

    def status(self):
        with self._lock:
            if self._process is None or self._process.poll() is not None:
                return {
                    "state": "disconnected",
                    "hostname": self.hostname or "",
                    "tailscale_ips": [],
                    "dns_name": "",
                    "proxy_state": "stopped",
                    "listen_port": 0,
                    "proxy_https": False,
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
                    "state": "disconnected", "hostname": self.hostname or "",
                    "tailscale_ips": [], "dns_name": "",
                    "proxy_state": "stopped", "listen_port": 0, "proxy_https": False,
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
                "state": "disconnected", "hostname": self.hostname or "",
                "tailscale_ips": [], "dns_name": "",
                "proxy_state": "stopped", "listen_port": 0,
            }

    def _atexit_disconnect(self):
        try:
            self.disconnect()
        except Exception:
            pass


def roleforge_receive(role):
    # Configuration and helper setup belong to Role.__init__; loading does not connect.
    return None
