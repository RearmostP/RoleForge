import tempfile
import threading
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock

from roleforge.builtin_roles.Tailscale.main import (
    TailscaleConfigurationError,
    _TailscaleProxy,
    _parse_body,
)


class TailscaleRoleConfigurationTests(unittest.TestCase):
    def role_input(self, body, path):
        return SimpleNamespace(
            body=body,
            source=SimpleNamespace(path=path, declaration_line=1),
            index=0,
            role_index=0,
        )

    def test_https_is_parsed_and_state_path_is_stable(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = Path(temporary) / "server.rfg"
            role = self.role_input(
                'connect.settings {\n'
                '  hostname = "home-gallery"\n'
                '  auth_key = "not-a-real-key"\n'
                '  state_dir = ".roleforge/tailscale"\n'
                '}\n'
                'proxy.settings {\n'
                '  https = true\n'
                '  target = "http://127.0.0.1:8000"\n'
                '  listen_port = 443\n'
                '}\n',
                config,
            )

            settings = _parse_body(role)

            self.assertTrue(settings["proxy"]["https"])
            self.assertEqual(
                settings["connect"]["state_dir"],
                str(Path(temporary) / ".roleforge" / "tailscale" / "roleforge-instance-0-0"),
            )

    def test_https_defaults_to_false_for_existing_configurations(self):
        with tempfile.TemporaryDirectory() as temporary:
            role = self.role_input(
                'connect.settings {\n'
                '  hostname = "home-gallery"\n'
                '  auth_key = "not-a-real-key"\n'
                '  state_dir = ".roleforge/tailscale"\n'
                '}\n',
                Path(temporary) / "server.rfg",
            )

            settings = _parse_body(role)

            self.assertFalse(settings["proxy"]["https"])

    def test_https_can_be_enabled_from_env_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            env_file = Path(temporary) / "tailscale.env"
            env_file.write_text("TAILSCALE_PROXY_HTTPS=true\n", encoding="utf-8")
            role = self.role_input(
                f'env_file = "{env_file}"\n'
                'connect.settings {\n'
                '  hostname = "home-gallery"\n'
                '  auth_key = "not-a-real-key"\n'
                '  state_dir = ".roleforge/tailscale"\n'
                '}\n'
                'proxy.settings {\n'
                '  https = false\n'
                '}\n',
                Path(temporary) / "server.rfg",
            )

            settings = _parse_body(role)

            self.assertTrue(settings["proxy"]["https"])

    def test_invalid_https_value_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            role = self.role_input(
                'connect.settings {\n'
                '  hostname = "home-gallery"\n'
                '  auth_key = "not-a-real-key"\n'
                '  state_dir = ".roleforge/tailscale"\n'
                '}\n'
                'proxy.settings {\n'
                '  https = yes\n'
                '}\n',
                Path(temporary) / "server.rfg",
            )

            with self.assertRaisesRegex(TailscaleConfigurationError, "must be true or false"):
                _parse_body(role)

    def test_proxy_start_passes_https_to_helper(self):
        role = SimpleNamespace(
            _lock=threading.RLock(),
            _process=SimpleNamespace(poll=lambda: None),
            settings={"connect": {}, "proxy": {}},
            proxy_settings={
                "target": "http://127.0.0.1:8000",
                "listen_port": "443",
                "https": True,
            },
            _request=Mock(return_value={"proxy_state": "running"}),
        )

        status = _TailscaleProxy(role).start()

        self.assertEqual(status["proxy_state"], "running")
        role._request.assert_called_once_with(
            {
                "command": "proxy_start",
                "target": "http://127.0.0.1:8000",
                "listen_port": 443,
                "https": True,
            },
            timeout=15,
        )
