# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

"""
Pytest configuration and fixtures for sovd-gateway integration tests.
"""

import asyncio
import asyncio.subprocess
import contextlib
import os
import re
import subprocess
import time
from collections.abc import AsyncGenerator
from pathlib import Path

import httpx
import pytest
import pytest_asyncio

# Configuration constants
_DEFAULT_URL = "http://127.0.0.1:0/sovd"
_CERT_VALIDITY_DAYS = 30
_GATEWAY_STARTUP_TIMEOUT = 15.0
_GATEWAY_SHUTDOWN_TIMEOUT = 5.0
_HTTP_CLIENT_TIMEOUT = 5.0
_HTTPS_CLIENT_TIMEOUT = 10.0  # HTTPS connections may need more time
_SUBPROCESS_READ_TIMEOUT = 0.5


def pytest_addoption(parser):
    """Add custom pytest command line options."""
    parser.addoption(
        "--sovd-gateway-bin",
        action="store",
        default=None,
        help="Path to sovd-gateway binary (defaults to 'cargo run --bin sovd-gateway')",
    )
    parser.addoption(
        "--sovd-gateway-args",
        action="store",
        default="",
        help="Additional arguments to pass to sovd-gateway (space-separated)",
    )
    parser.addoption(
        "--sovd-gateway-profile",
        action="store",
        default="release",
        help="Cargo profile to use when building with 'cargo run --profile' (default: release)",
    )
    parser.addoption(
        "--sovd-gateway-features",
        action="store",
        default="",
        help="Comma-separated list of features to pass to 'cargo run --features'",
    )


@pytest.fixture
def gateway_binary(request) -> str | None:
    """Get the path to sovd-gateway binary or None to use cargo run."""
    return request.config.getoption("--sovd-gateway-bin")


@pytest.fixture
def gateway_args(request) -> list[str]:
    """Get default additional arguments for sovd-gateway."""
    args_str = request.config.getoption("--sovd-gateway-args")
    return args_str.split() if args_str.strip() else ["--url", _DEFAULT_URL]


@pytest.fixture
def gateway_profile(request) -> str:
    """Get the cargo profile to use for building."""
    return request.config.getoption("--sovd-gateway-profile")


@pytest.fixture
def gateway_features(request) -> list[str]:
    """Get the list of features to use for cargo build."""
    features_str = request.config.getoption("--sovd-gateway-features")
    return features_str.split(",") if features_str.strip() else []


@pytest.fixture
def gateway_env() -> dict[str, str]:
    """Get default environment variables for sovd-gateway."""
    return dict(os.environ)


@pytest.fixture(scope="session")
def project_root() -> Path:
    """Get the project root directory."""
    return Path(__file__).parent.parent


@pytest.fixture(scope="session")
def openssl(project_root: Path, tmp_path_factory) -> dict[str, Path]:
    """Generate certificates using mkcerts.sh and return a dictionary mapping cert types to file paths."""
    temp_dir = tmp_path_factory.mktemp("sovd-certs")
    mkcerts_script = project_root / "scripts" / "mkcerts.sh"

    try:
        subprocess.run(
            [
                str(mkcerts_script),
                str(temp_dir),
                str(_CERT_VALIDITY_DAYS),
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
        self.process: asyncio.subprocess.Process | None = None
        self.base_url: str | None = None
        self._use_https: bool = False  # Track if HTTPS is being used

    async def start(self, args: list[str] | None = None, env: dict[str, str] | None = None) -> str:
        """
        Start the gateway server and return the URL.

        Args:
            args: Additional arguments to pass to the gateway
            env: Environment variables to set

        Returns:
            The base URL of the started server
        """
        cmd_args = args or []
        process_env = env or os.environ.copy()

        if self.binary_path:
            # Use pre-built binary
            cmd = [self.binary_path] + cmd_args
        else:
            # Build first (without timeout to avoid build interruption)
            await self._build_binary(process_env)

            # Then run the built binary
            cmd = ["cargo", "run", "--bin", "sovd-gateway", "--profile", self.profile]
            if self.features:
                cmd.extend(["--features", ",".join(self.features)])
            cmd.extend(["--"] + cmd_args)

        self.process = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            cwd=self.project_root,
            env=process_env,
        )

        self.base_url = await self._wait_for_ready()
        return self.base_url

    async def stop(self, timeout: float = _GATEWAY_SHUTDOWN_TIMEOUT) -> None:
        """Stop the gateway gracefully."""
        if self.process is None:
            return

        try:
            # Send SIGTERM first
            self.process.terminate()
            try:
                await asyncio.wait_for(self.process.wait(), timeout=timeout)
            except TimeoutError:
                # Force kill if graceful shutdown fails
                self.process.kill()
                await self.process.wait()
        finally:
            self.process = None
            self.base_url = None

    def __del__(self):
        """Cleanup on deletion to prevent zombie processes."""
        if self.process and self.is_running():
            with contextlib.suppress(ProcessLookupError):
                self.process.kill()

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
        return self.process.returncode is None

    def get_pid(self) -> int | None:
        """Get the process ID of the running gateway."""
        if self.process is None:
            return None
        return self.process.pid

    async def _build_binary(self, env: dict[str, str]) -> None:
        """
        Build the binary before running it.

        This method runs 'cargo build' without a timeout to avoid interrupting
        the build process, which can be slow on first run or after clean.

        Args:
            env: Environment variables to use for the build

        Raises:
            RuntimeError: If the build fails
        """
        build_cmd = ["cargo", "build", "--bin", "sovd-gateway", "--profile", self.profile]
        if self.features:
            build_cmd.extend(["--features", ",".join(self.features)])

        # Run build without timeout (can be slow on first build)
        build_process = await asyncio.create_subprocess_exec(
            *build_cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self.project_root,
            env=env,
        )

        # Wait for build to complete
        stdout, stderr = await build_process.communicate()

        if build_process.returncode != 0:
            error_msg = stderr.decode("utf-8", errors="replace") if stderr else "Unknown error"
            msg = f"Cargo build failed with exit code {build_process.returncode}: {error_msg}"
            raise RuntimeError(msg)

    async def _wait_for_ready(self, timeout: float = _GATEWAY_STARTUP_TIMEOUT) -> str:
        """
        Wait for the gateway to be ready and extract the URL from logs.

        Returns:
            The base URL extracted from logs

        Raises:
            RuntimeError: If process exits or timeout occurs
        """
        if self.process is None:
            msg = "Process not started"
            raise RuntimeError(msg)

        start_time = time.time()
        # Pattern to match the gateway endpoint log with quoted values
        # Example: protocol="http" listen_addr="127.0.0.1:8080" base_path="/sovd"
        # Example: protocol="https" listen_addr="[::1]:8443" base_path="/sovd"
        # Simplified: just match quoted strings
        listen_pattern = re.compile(r'protocol="([^"]+)".*listen_addr="([^"]+)".*base_path="([^"]+)"')

        while time.time() - start_time < timeout:
            # Check if process exited
            if self.process.returncode is not None:
                msg = f"Gateway process exited with code {self.process.returncode}"
                raise RuntimeError(msg)

            # Ensure stdout is available
            if self.process.stdout is None:
                msg = "Process stdout not available"
                raise RuntimeError(msg)

            try:
                # Read with short timeout to allow checking overall timeout
                line_bytes = await asyncio.wait_for(
                    self.process.stdout.readline(),
                    timeout=_SUBPROCESS_READ_TIMEOUT,
                )

                # Empty bytes means EOF
                if not line_bytes:
                    if self.process.returncode is not None:
                        msg = "Process exited unexpectedly"
                        raise RuntimeError(msg)
                    continue

                line = line_bytes.decode("utf-8", errors="replace").rstrip()

                # Check for gateway endpoint log
                match = listen_pattern.search(line)
                if match:
                    protocol = match.group(1)  # http or https
                    listen_addr = match.group(2)  # host:port (e.g., "127.0.0.1:8080" or "[::1]:8443")
                    base_path = match.group(3)

                    # Convert loopback addresses to localhost for SSL certificate compatibility
                    # Certificates have DNS:localhost in SAN, not IP:127.0.0.1 or IP:::1
                    if listen_addr.startswith(("[::1]:", "127.0.0.1:")):
                        # Extract port and use localhost
                        port = listen_addr.split(":")[-1]
                        listen_addr = f"localhost:{port}"

                    return f"{protocol}://{listen_addr}{base_path}"

            except TimeoutError:
                # No output in this interval, continue loop
                continue
            except OSError as e:
                print(f"Warning: Error reading process output: {e}")
                continue

        msg = f"Gateway failed to start within {timeout} seconds"
        raise RuntimeError(msg)


@pytest_asyncio.fixture
async def gateway_manager(
    gateway_binary: str | None,
    project_root: Path,
    gateway_profile: str,
    gateway_features: list[str],
) -> AsyncGenerator[GatewayManager, None]:
    manager = GatewayManager(gateway_binary, project_root, gateway_profile, gateway_features)
    yield manager

    if manager.is_running():
        await manager.stop()


@pytest_asyncio.fixture
async def gateway(
    gateway_manager: GatewayManager,
    gateway_args: list[str],
    gateway_env: dict[str, str],
) -> AsyncGenerator[GatewayManager, None]:
    await gateway_manager.start(args=gateway_args, env=gateway_env)
    yield gateway_manager
    await gateway_manager.stop()


@pytest_asyncio.fixture
async def gateway_url(gateway) -> str:
    """Gateway base URL."""
    return gateway.base_url


@pytest_asyncio.fixture
async def client(gateway_url: str) -> AsyncGenerator[httpx.AsyncClient, None]:
    try:
        async with httpx.AsyncClient(base_url=gateway_url, timeout=_HTTP_CLIENT_TIMEOUT) as client:
            yield client
    except httpx.RequestError as e:
        pytest.skip(f"Could not reach endpoint at {gateway_url}: {e}")
