# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0
#

"""
Pytest configuration and fixtures for UI tests using Playwright.
"""

from collections.abc import AsyncGenerator

import pytest
import pytest_asyncio
from playwright.async_api import Page, async_playwright

pytestmark = pytest.mark.asyncio


@pytest.fixture
def gateway_features() -> list[str]:
    return ["ui"]


@pytest_asyncio.fixture
async def page() -> AsyncGenerator[Page, None]:
    """
    Create a Playwright page with browser and context.

    Browser launch arguments are configured for headless CI environments:
    - --no-sandbox: Required in containerized/CI environments where sandboxing
      is handled at a different level (e.g., Docker, GitHub Actions)
    - --disable-dev-shm-usage: Prevents /dev/shm exhaustion in containers with
      limited shared memory
    - --disable-web-security: Allows testing without CORS restrictions (test-only)

    Note: These flags reduce security and should NEVER be used in production.
    """
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            args=[
                "--no-sandbox",  # CI/container requirement
                "--disable-dev-shm-usage",  # CI/container requirement
                "--disable-web-security",  # Test convenience only
            ]
        )
        context = await browser.new_context(ignore_https_errors=True)
        page = await context.new_page()
        yield page
        await context.close()
        await browser.close()


@pytest_asyncio.fixture
async def ui(page: Page, gateway_url: str) -> AsyncGenerator[Page, None]:
    """Navigate to the UI and wait for it to be ready."""
    ui_url = gateway_url.replace("/sovd", "/ui")
    await page.goto(ui_url, wait_until="domcontentloaded")
    await page.wait_for_selector("#app", state="attached")
    yield page
