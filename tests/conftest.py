# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

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

_DEFAULT_URL = "http://127.0.0.1:0/sovd"
_CERT_VALIDITY_DAYS = 30
_GATEWAY_STARTUP_TIMEOUT = 15.0
_GATEWAY_SHUTDOWN_TIMEOUT = 5.0
_HTTP_CLIENT_TIMEOUT = 5.0
_HTTPS_CLIENT_TIMEOUT = 10.0
_SUBPROCESS_READ_TIMEOUT = 0.5


def pytest_addoption(parser):
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
    return request.config.getoption("--sovd-gateway-bin")


@pytest.fixture
def gateway_args(request) -> list[str]:
    args_str = request.config.getoption("--sovd-gateway-args")
    return args_str.split() if args_str.strip() else ["--url", _DEFAULT_URL]


@pytest.fixture
def gateway_profile(request) -> str:
    return request.config.getoption("--sovd-gateway-profile")


@pytest.fixture
def gateway_features(request) -> list[str]:
    features_str = request.config.getoption("--sovd-gateway-features")
    return features_str.split(",") if features_str.strip() else []


@pytest.fixture
def gateway_env() -> dict[str, str]:
    return os.environ.copy()


@pytest.fixture(scope="session")
def project_root() -> Path:
    return Path(__file__).parent.parent


@pytest.fixture(scope="session")
def openssl(project_root: Path, tmp_path_factory) -> dict[str, Path]:
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
        stderr = f", stderr: {e.stderr.strip()}" if e.stderr else ""
        pytest.fail(f"Failed to generate certificates (exit code {e.returncode}){stderr}")

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

    async def start(self, args: list[str] | None = None, env: dict[str, str] | None = None) -> str:
        cmd_args = args or []
        process_env = env or os.environ.copy()

        if self.binary_path:
            cmd = [self.binary_path, *cmd_args]
        else:
            await self._build_binary(process_env)

            cmd = ["cargo", "run", "--bin", "sovd-gateway", "--profile", self.profile]
            if self.features:
                cmd.extend(["--features", ",".join(self.features)])
            cmd.extend(["--", *cmd_args])

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
        if self.process is None:
            return

        try:
            self.process.terminate()
            try:
                await asyncio.wait_for(self.process.wait(), timeout=timeout)
            except TimeoutError:
                self.process.kill()
                await self.process.wait()
        finally:
            self.process = None
            self.base_url = None

    def __del__(self):
        if self.process and self.is_running():
            with contextlib.suppress(ProcessLookupError):
                self.process.kill()

    def send_signal(self, sig: int) -> None:
        if self.process is None:
            msg = "Gateway is not running"
            raise RuntimeError(msg)
        self.process.send_signal(sig)

    def is_running(self) -> bool:
        return self.process is not None and self.process.returncode is None

    def get_pid(self) -> int | None:
        return self.process.pid if self.process else None

    async def _build_binary(self, env: dict[str, str]) -> None:
        build_cmd = ["cargo", "build", "--bin", "sovd-gateway", "--profile", self.profile]
        if self.features:
            build_cmd.extend(["--features", ",".join(self.features)])

        build_process = await asyncio.create_subprocess_exec(
            *build_cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self.project_root,
            env=env,
        )

        _, stderr = await build_process.communicate()

        if build_process.returncode != 0:
            error_msg = stderr.decode("utf-8", errors="replace") if stderr else "Unknown error"
            msg = f"Cargo build failed with exit code {build_process.returncode}: {error_msg}"
            raise RuntimeError(msg)

    async def _wait_for_ready(self, timeout: float = _GATEWAY_STARTUP_TIMEOUT) -> str:
        if self.process is None:
            msg = "Process not started"
            raise RuntimeError(msg)

        start_time = time.time()
        listen_pattern = re.compile(
            r'protocol="(?P<protocol>[^"]+)".*listening="(?P<addr>[^"]+)".*base="(?P<base>[^"]+)"'
        )

        while time.time() - start_time < timeout:
            if self.process.returncode is not None:
                msg = f"Gateway process exited with code {self.process.returncode}"
                raise RuntimeError(msg)

            if self.process.stdout is None:
                msg = "Process stdout not available"
                raise RuntimeError(msg)

            try:
                line_bytes = await asyncio.wait_for(
                    self.process.stdout.readline(),
                    timeout=_SUBPROCESS_READ_TIMEOUT,
                )

                if not line_bytes:
                    if self.process.returncode is not None:
                        msg = "Process exited unexpectedly"
                        raise RuntimeError(msg)
                    continue

                line = line_bytes.decode("utf-8", errors="replace").rstrip()

                if match := listen_pattern.search(line):
                    return f"{match['protocol']}://{match['addr']}{match['base']}"

            except (TimeoutError, OSError):
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
    return gateway.base_url


@pytest_asyncio.fixture
async def client(gateway_url: str) -> AsyncGenerator[httpx.AsyncClient, None]:
    async with httpx.AsyncClient(base_url=gateway_url, timeout=_HTTP_CLIENT_TIMEOUT) as client:
        yield client
