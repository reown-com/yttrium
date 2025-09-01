import test, { expect } from "@playwright/test";

test('self-connect Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    await page.getByTestId("connect-button").click();
    await page.getByTestId("self-connect-button").click();
    await page.getByTestId("approve-button").click();
    await expect(page.locator('ul', { hasText: 'Session' })).toBeVisible();
});
