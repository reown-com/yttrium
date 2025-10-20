import { test, expect, type Page } from '@playwright/test';

test('connect Rust app to JS wallet', async ({ browser, page, context, baseURL }) => {
    await page.goto(baseURL!);
    await page.getByTestId("connect-button").click();
    await expect(page.getByTestId("pairing-uri")).toBeVisible();
    await expect(page.getByTestId("pairing-uri")).toHaveAttribute("data-pairing-uri");
    const uri = await page.getByTestId("pairing-uri").getAttribute("data-pairing-uri");

    const wallet = await context.newPage();
    await wallet.goto("https://react-wallet.walletconnect.com/walletconnect");
    await expect(wallet.getByTestId("uri-input")).toBeVisible();
    await expect(wallet.getByTestId("uri-input")).toBeEditable();
    await wallet.getByTestId("uri-input").fill(uri!);
    await wallet.getByTestId("uri-connect-button").click({ /*force: true*/ });
});

// TODO add tests for Rust app and JS wallet
// TODO test verify in various scenarios

// test('sign message Rust wallet to JS app', async ({ browser, page, baseURL }) => {
//     await page.goto(baseURL!);
//     const context = await browser.newContext();
//     const app = await context.newPage();
//     await connectJsApp(app, page);
//     await app.getByTestId("sign-message-button").click();
//     await page.getByTestId('request-approve-button').click();
//     await expect(page.getByText("Signature approved")).toBeVisible();
//     await expect(app.getByText("Signing Succeeded")).toBeVisible();
// });
