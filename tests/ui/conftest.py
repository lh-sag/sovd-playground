# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

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
    async with async_playwright() as p:
        browser = await p.chromium.launch(args=["--no-sandbox", "--disable-dev-shm-usage", "--disable-web-security"])
        context = await browser.new_context(ignore_https_errors=True)
        page = await context.new_page()
        yield page


@pytest_asyncio.fixture
async def ui(page: Page, gateway_url: str) -> AsyncGenerator[Page, None]:
    ui_url = gateway_url.replace("/sovd", "/ui")
    await page.goto(ui_url, wait_until="domcontentloaded")
    await page.wait_for_selector("#app", state="attached")
    yield page
