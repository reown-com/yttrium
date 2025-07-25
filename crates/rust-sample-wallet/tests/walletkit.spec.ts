import { test, expect, type Page } from '@playwright/test';

async function connect(app: Page, page: Page, baseURL: string) {
    await app.goto("https://appkit-lab.reown.com/library/wagmi/");
    await app.getByTestId("connect-button").click({ force: true });
    await app.getByTestId("wallet-selector-walletconnect").click();
    const qr = app.getByTestId("wui-qr-code");
    await expect(qr).toBeVisible();
    const uri = await qr.getAttribute("uri");

    await page.goto(baseURL!);
    const pairingUri = page.locator('#pairing-uri');
    await expect(pairingUri).toBeVisible();
    await pairingUri.fill(uri!);
    await page.waitForTimeout(100);
    await page.locator('button', { hasText: 'Pair' }).click();
    await expect(page.getByText("Pairing approved")).toBeVisible();
    await expect(page.getByText("Session")).toBeVisible();
    expect(await app.getByTestId("w3m-caip-address").innerText()).toEqual("eip155:1:0x0000000000000000000000000000000000000000");
}

test('connect', async ({ browser, page, baseURL }) => {
    const context = await browser.newContext();
    const app = await context.newPage();
    await connect(app, page, baseURL!);
});

test('sign', async ({ browser, page, baseURL }) => {
    const context = await browser.newContext();
    const app = await context.newPage();
    await connect(app, page, baseURL!);
    await app.getByTestId("sign-message-button").click();
    await page.locator('button', { hasText: 'Approve' }).click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
});
