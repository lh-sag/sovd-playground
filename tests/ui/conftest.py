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

from pathlib import Path

import pytest
from playwright.sync_api import Browser, Page, Playwright


@pytest.fixture(scope="module")
def gateway_features() -> list[str]:
    """Override parent fixture to ensure UI feature is enabled."""
    return ["ui"]


@pytest.fixture(scope="session")
def browser(playwright: Playwright) -> Browser:
    """Browser instance for the test session."""
    import os

    # Allow customization via environment variables
    headless = os.environ.get("PYTEST_BROWSER_HEADLESS", "true").lower() == "true"
    browser_args = [
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--disable-web-security",
    ]

    # Add additional args from environment if provided
    if extra_args := os.environ.get("PYTEST_BROWSER_ARGS"):
        browser_args.extend(extra_args.split())

    try:
        browser = playwright.chromium.launch(
            headless=headless,
            args=browser_args,
        )
        yield browser
        browser.close()
    except OSError as e:
        pytest.fail(f"Failed to launch browser: {e}. Run 'uv run playwright install' to install browser dependencies.")


@pytest.fixture
def page(browser: Browser) -> Page:
    """Page instance for each test."""
    context = browser.new_context(ignore_https_errors=True)
    page = context.new_page()
    yield page
    context.close()


@pytest.fixture
def gateway_url(gateway) -> str:
    """Gateway base URL for UI tests."""
    return gateway.base_url


@pytest.fixture
def check_playwright_installed():
    """Check if Playwright browsers are installed."""
    import platform

    # Handle different OS cache locations
    if platform.system() == "Windows":
        cache_paths = [
            Path.home() / "AppData" / "Local" / "ms-playwright",
        ]
    elif platform.system() == "Darwin":  # macOS
        cache_paths = [
            Path.home() / "Library" / "Caches" / "ms-playwright",
        ]
    else:  # Linux and others
        cache_paths = [
            Path.home() / ".cache" / "ms-playwright",
        ]

    for cache_path in cache_paths:
        if cache_path.exists() and any(d.name.startswith("chromium") for d in cache_path.iterdir() if d.is_dir()):
            return  # Found it

    pytest.skip("Playwright browsers not installed. Run 'uv run playwright install' to install browser dependencies.")
