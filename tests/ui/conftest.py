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
Pytest configuration and fixtures for UI tests using Playwright.
"""

import time
from collections.abc import Generator
from pathlib import Path

import pytest
import requests
from playwright.sync_api import Browser, BrowserContext, Page, Playwright


@pytest.fixture(scope="session")
def playwright() -> Generator[Playwright, None, None]:
    """Playwright instance for the test session."""
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        yield p


@pytest.fixture(scope="session")
def browser(playwright: Playwright) -> Generator[Browser, None, None]:
    """Browser instance for the test session."""
    try:
        browser = playwright.chromium.launch(
            headless=True,
            args=[
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-web-security",
                "--disable-features=VizDisplayCompositor",
            ],
        )
        yield browser
        browser.close()
    except Exception as e:  # noqa: BLE001
        pytest.skip(f"Playwright browsers not available: {e}. Run 'uv run playwright install' to fix.")


@pytest.fixture
def context(browser: Browser) -> Generator[BrowserContext, None, None]:
    """Browser context for each test."""
    context = browser.new_context(
        viewport={"width": 1280, "height": 720},
        ignore_https_errors=True,
    )
    yield context
    context.close()


@pytest.fixture
def page(context: BrowserContext) -> Generator[Page, None, None]:
    """Page instance for each test."""
    page = context.new_page()
    yield page
    page.close()


@pytest.fixture
def gateway_url(gateway) -> str:
    """Gateway base URL for UI tests."""
    return gateway.base_url


@pytest.fixture
def check_playwright_installed():
    """Check if Playwright browsers are installed."""
    # Check for common Playwright browser installation paths
    playwright_cache = Path("~/.cache/ms-playwright").expanduser()
    if not playwright_cache.exists():
        pytest.skip("Playwright browsers not installed. Run 'uv run playwright install' to install.")

    # Check for chromium specifically
    chromium_dirs = [d for d in playwright_cache.iterdir() if d.name.startswith("chromium")]
    if not chromium_dirs:
        pytest.skip("Chromium browser not installed for Playwright. Run 'uv run playwright install' to install.")


@pytest.fixture
def ui_url(gateway_url: str) -> str:
    """UI base URL (uses /ui path for embedded UI)."""
    # Extract base URL without the /opensovd path and add /ui
    base_url = gateway_url.replace("/opensovd", "")
    return f"{base_url}/ui"


@pytest.fixture
def wait_for_gateway_ready(gateway_url: str) -> None:
    """Wait for gateway to be ready before running UI tests."""
    max_retries = 30
    retry_delay = 1.0

    for attempt in range(max_retries):
        try:
            response = requests.get(f"{gateway_url}/opensovd/version-info", timeout=5)
            if response.status_code == 200:
                return
        except (requests.RequestException, requests.ConnectionError):
            pass

        if attempt < max_retries - 1:
            time.sleep(retry_delay)

    pytest.fail(f"Gateway at {gateway_url} is not ready after {max_retries} attempts")


@pytest.fixture
def mock_components_data():
    """Mock component data for testing."""
    return {
        "items": [
            {
                "id": "engine-controller",
                "name": "Engine Control Unit",
                "data": "/opensovd/components/engine-controller/data",
                "data-list": "/opensovd/components/engine-controller/data-list",
                "faults": "/opensovd/components/engine-controller/faults",
            },
            {
                "id": "transmission-controller",
                "name": "Transmission Control Unit",
                "data": "/opensovd/components/transmission-controller/data",
                "configurations": "/opensovd/components/transmission-controller/configurations",
            },
            {
                "id": "brake-controller",
                "name": "Brake Control Unit",
                "data": "/opensovd/components/brake-controller/data",
                "operations": "/opensovd/components/brake-controller/operations",
            },
        ]
    }


@pytest.fixture
def mock_version_info():
    """Mock version info for testing."""
    return {
        "sovd_info": [
            {
                "version": "1.1",
                "base_uri": "/opensovd/v1",
                "vendor_info": {
                    "vendor": "Liebherr",
                    "product": "OpenSOVD Gateway",
                    "version": "0.1.0",
                },
            },
            {
                "version": "1.2",
                "base_uri": "/opensovd/v2",
                "vendor_info": {
                    "vendor": "Liebherr",
                    "product": "OpenSOVD Gateway",
                    "version": "0.1.0",
                    "experimental": True,
                },
            },
        ]
    }


@pytest.fixture
def loaded_ui_page(page: Page, ui_url: str, wait_for_gateway_ready, check_playwright_installed) -> Page:
    """Page with UI loaded and ready."""
    page.goto(ui_url)

    # Wait for Vue app to be mounted
    page.wait_for_selector("#app", timeout=10000)

    # Wait for any initial loading to complete
    page.wait_for_timeout(1000)

    return page
