#
# Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
#

"""
Pytest configuration and fixtures for osovd-gateway integration tests.
"""

import contextlib
import re
import subprocess
import time
from collections.abc import Generator
from pathlib import Path

import pytest
import sh
from sh import ErrorReturnCode

_DEFAULT_URL = "http://127.0.0.1:0/opensovd"


def pytest_addoption(parser):
    """Add custom pytest command line options."""
    parser.addoption(
        "--osovd-gateway-bin",
        action="store",
        default=None,
        help="Path to osovd-gateway binary (defaults to 'cargo run --bin osovd-gateway')",
    )
    parser.addoption(
        "--osovd-gateway-args",
        action="store",
        default="",
        help="Additional arguments to pass to osovd-gateway (space-separated)",
    )
    parser.addoption(
        "--osovd-gateway-profile",
        action="store",
        default="release",
        help="Cargo profile to use when building with 'cargo run --profile' (default: release)",
    )
    parser.addoption(
        "--osovd-gateway-features",
        action="store",
        default="",
        help="Comma-separated list of features to pass to 'cargo run --features'",
    )


@pytest.fixture(scope="module")
def gateway_binary(request) -> str | None:
    """Get the path to osovd-gateway binary or None to use cargo run."""
    return request.config.getoption("--osovd-gateway-bin")


@pytest.fixture(scope="module")
def gateway_args(request) -> list[str]:
    """Get default additional arguments for osovd-gateway."""
    args_str = request.config.getoption("--osovd-gateway-args")
    return args_str.split() if args_str.strip() else ["--url", _DEFAULT_URL]


@pytest.fixture(scope="module")
def gateway_profile(request) -> str:
    """Get the cargo profile to use for building."""
    return request.config.getoption("--osovd-gateway-profile")


@pytest.fixture(scope="module")
def gateway_features(request) -> list[str]:
    """Get the list of features to use for cargo build."""
    features_str = request.config.getoption("--osovd-gateway-features")
    return features_str.split(",") if features_str.strip() else []


@pytest.fixture
def gateway_env() -> dict[str, str]:
    """Get default environment variables for osovd-gateway."""
    import os

    return dict(os.environ)


@pytest.fixture(scope="session")
def project_root() -> Path:
    """Get the project root directory."""
    return Path(__file__).parent.parent


@pytest.fixture
def openssl(project_root: Path, tmp_path) -> dict[str, Path]:
    """Generate certificates using mkcerts.sh and return a dictionary mapping cert types to file paths."""
    temp_dir = tmp_path / "osovd-certs"
    temp_dir.mkdir()
    mkcerts_script = project_root / "scripts" / "mkcerts.sh"

    try:
        subprocess.run(
            [
                str(mkcerts_script),
                str(temp_dir),
                "30",  # Default to 30 days, consider making configurable
                "--no-verify",
            ],
            check=True,
            capture_output=True,
            text=True,
            cwd=project_root,
        )
    except subprocess.CalledProcessError as e:
        error_details = f"exit code {e.returncode}"
        if e.stderr:
            error_details += f", stderr: {e.stderr.strip()}"
        pytest.fail(f"Failed to generate certificates ({error_details})")

    # Create certificate dictionary mapping cert types to file paths
    cert_files = {
        "ca_cert": temp_dir / "ca-cert.pem",
        "server_cert": temp_dir / "server-cert.pem",
        "server_key": temp_dir / "server-key.pem",
        "client_cert": temp_dir / "client-cert.pem",
        "client_key": temp_dir / "client-key.pem",
    }

    for cert_type, cert_path in cert_files.items():
        if not cert_path.exists():
            pytest.fail(f"Required certificate file not found: {cert_type} -> {cert_path}")

    return cert_files


class GatewayManager:
    def __init__(
        self,
        binary_path: str | None,
        project_root: Path,
        profile: str = "release",
        features: list[str] | None = None,
    ):
        self.binary_path = binary_path
        self.project_root = project_root
        self.profile = profile
        self.features = features or []
        self.process: sh.RunningCommand | None = None
        self.base_url: str | None = None

    def start(self, args: list[str] | None = None, env: dict[str, str] | None = None) -> str:
        """
        Start the gateway server and return the URL.

        Args:
            args: Additional arguments to pass to the gateway
            env: Environment variables to set

        Returns:
            The base URL of the started server
        """
        cmd_args = args or []
        process_env = env or {}

        try:
            if self.binary_path:
                # Use pre-built binary
                cmd = sh.Command(self.binary_path)
                self.process = cmd(
                    *cmd_args,
                    _cwd=self.project_root,
                    _env=process_env,
                    _bg=True,
                    _bg_exc=False,
                    _iter=True,
                    _err_to_out=True,
                )
            else:
                # Use cargo run with profile and features
                cargo = sh.Command("cargo")
                cargo_args = ["run", "--bin", "osovd-gateway", "--profile", self.profile]
                if self.features:
                    cargo_args.extend(["--features", ",".join(self.features)])
                cargo_args.extend(["--"] + cmd_args)
                self.process = cargo(
                    *cargo_args,
                    _cwd=self.project_root,
                    _env=process_env,
                    _bg=True,
                    _bg_exc=False,
                    _iter=True,
                    _err_to_out=True,
                )

            self.base_url = self._wait_for_ready()
            return self.base_url

        except ErrorReturnCode as e:
            msg = f"Failed to start gateway: {e}"
            raise RuntimeError(msg) from e

    def stop(self, timeout: float = 5.0) -> None:
        """Stop the gateway gracefully."""
        if self.process is None:
            return

        try:
            # Send SIGTERM first
            self.process.terminate()
            try:
                self.process.wait(timeout=timeout)
            except sh.TimeoutException:
                # Force kill if graceful shutdown fails
                self.process.kill()
                with contextlib.suppress(sh.SignalException_SIGKILL, sh.SignalException_SIGTERM):
                    self.process.wait()
        except (sh.SignalException_SIGKILL, sh.SignalException_SIGTERM):
            # Expected when process is terminated
            pass
        finally:
            self.process = None
            self.base_url = None

    def __del__(self):
        """Cleanup on deletion to prevent zombie processes."""
        if self.process and self.is_running():
            import contextlib

            with contextlib.suppress(Exception):
                self.stop()

    def send_signal(self, sig: int) -> None:
        """Send a signal to the running gateway process."""
        if self.process is None:
            msg = "Gateway is not running"
            raise RuntimeError(msg)
        self.process.send_signal(sig)

    def is_running(self) -> bool:
        """Check if the gateway process is still running."""
        if self.process is None:
            return False
        return self.process.is_alive()

    def get_pid(self) -> int | None:
        """Get the process ID of the running gateway."""
        if self.process is None:
            return None
        return self.process.pid

    def __iter__(self):
        """Make GatewayManager iterable over log lines."""
        return self.process

    def _wait_for_ready(self, timeout: float = 15.0) -> str:
        """
        Wait for the gateway to be ready and extract the URL from logs.

        Returns:
            The base URL extracted from logs
        """
        start_time = time.time()
        # Pattern to match the actix server listening log
        listen_pattern = re.compile(r"listening on: ([0-9.]+):(\d+)")
        # Pattern to detect HTTPS/TLS configuration
        tls_pattern = re.compile(r"(TLS|SSL|https|certificate)", re.IGNORECASE)
        protocol = "http"  # Default to HTTP

        while time.time() - start_time < timeout:
            if not self.is_running():
                logs = "\n".join(self)
                msg = f"Gateway process exited unexpectedly. Logs:\n{logs}"
                raise RuntimeError(msg)

            try:
                if self.process is not None:
                    for line in self.process:
                        line = line.rstrip("\n\r")
                        # Check if TLS/HTTPS is enabled
                        if tls_pattern.search(line):
                            protocol = "https"
                        protocol = "http"
                        match = listen_pattern.search(line)
                        if match:
                            host = match.group(1)
                            port = match.group(2)
                            return f"{protocol}://{host}:{port}/opensovd"
            except ValueError as e:
                # Log the error for debugging but continue waiting
                print(f"Warning: Error reading process output: {e}")
                continue

            time.sleep(0.1)

        msg = f"Gateway failed to start within {timeout} seconds."
        raise RuntimeError(msg)


@pytest.fixture
def gateway_manager(
    gateway_binary: str | None,
    project_root: Path,
    gateway_profile: str,
    gateway_features: list[str],
) -> Generator[GatewayManager, None, None]:
    manager = GatewayManager(gateway_binary, project_root, gateway_profile, gateway_features)
    yield manager

    if manager.is_running():
        manager.stop()


@pytest.fixture
def gateway(
    gateway_manager: GatewayManager,
    gateway_args: list[str],
    gateway_env: dict[str, str],
) -> Generator[GatewayManager, None, None]:
    gateway_manager.start(args=gateway_args, env=gateway_env)
    yield gateway_manager
    gateway_manager.stop()
