# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

from playwright.async_api import expect


async def test_page_loads(ui):
    await expect(ui).to_have_title("SOVD")
    await expect(ui.locator("#app")).to_be_visible()
    await expect(ui.locator(".topbar")).to_be_visible()
    await expect(ui.locator(".sidebar")).to_be_visible()
    await expect(ui.locator(".main-panel")).to_be_visible()
