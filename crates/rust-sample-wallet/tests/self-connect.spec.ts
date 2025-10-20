import test, { expect } from "@playwright/test";

test('self-connect Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    await expect(page.getByTestId("app-sessions").locator('li')).toHaveCount(0);
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(0);
    await page.getByTestId("connect-button").click();
    await page.getByTestId("self-connect-button").click();
    await expect(page.getByText(`VerifyContext { origin: Some("${baseURL}"), validation: Valid, is_scam: false }`)).toBeVisible();
    await page.getByTestId("pairing-approve-button").click();
    await expect(page.getByTestId("app-sessions").locator('li')).toHaveCount(1);
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(1);
});
