import { test, expect, type Page } from '@playwright/test';

async function connectJsApp(app: Page, page: Page) {
    await app.goto("https://lab.reown.com/appkit/?name=wagmi");
    await app.getByTestId("connect-button").click({ force: true });
    await app.getByTestId("wallet-selector-walletconnect").click();
    const qr = app.getByTestId("wui-qr-code");
    await expect(qr).toBeVisible();
    const uri = await qr.getAttribute("uri");

    const pairingUri = page.getByTestId("input-pairing-uri").locator('input');
    await expect(pairingUri).toBeVisible();
    await pairingUri.fill(uri!);
    await page.getByTestId('pair-submit-button').click();
    await expect(page.getByText('VerifyContext { origin: Some("https://lab.reown.com"), validation: Valid, is_scam: false }')).toBeVisible();
    await page.getByTestId('pairing-approve-button').click();
    await expect(page.getByText("Pairing approved")).toBeVisible();
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(1);
    await expect(app.getByTestId("w3m-caip-address")).toHaveText("eip155:1:0x0000000000000000000000000000000000000000");
}

test('connect Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
});

test('sign message Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    await app.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
});

test.skip("receives sign after refresh Rust wallet to JS app", async ({ browser, page, baseURL }) => {
    // disabled because session queue not implemented yet
    await page.goto(baseURL!);
    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    await app.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
    await app.locator("#toast-close-button").click();
    await expect(app.getByText("Signing Succeeded")).not.toBeVisible();

    await page.reload();

    await app.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
});

test("high latency Rust wallet to JS app", async ({ browser, baseURL }) => {
    const walletContext = await browser.newContext();
    const page = await walletContext.newPage();
    await page.goto(baseURL!);
    await expect(page.getByTestId("input-pairing-uri").locator('input')).toBeVisible();
    const walletCDPSession = await walletContext.newCDPSession(page);
    walletCDPSession.send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 2000,
        downloadThroughput: -1,
        uploadThroughput: -1,
    });

    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    await app.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
});

test("low bandwidth Rust wallet to JS app", async ({ browser, baseURL }) => {
    const walletContext = await browser.newContext();
    const page = await walletContext.newPage();
    await page.goto(baseURL!);
    await expect(page.getByTestId("input-pairing-uri").locator('input')).toBeVisible();
    const walletCDPSession = await walletContext.newCDPSession(page);
    walletCDPSession.send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 0,
        downloadThroughput: 100000,
        uploadThroughput: 100000,
    });

    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    await app.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app.getByText("Signing Succeeded")).toBeVisible();
});

test("retry pairing after offline Rust wallet to JS app", async ({ browser, baseURL }) => {
    const walletContext = await browser.newContext();
    const page = await walletContext.newPage();
    await page.goto(baseURL!);
    await expect(page.getByTestId("input-pairing-uri").locator('input')).toBeVisible();
    const walletCDPSession = await walletContext.newCDPSession(page);
    walletCDPSession.send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 0,
        downloadThroughput: -1,
        uploadThroughput: -1,
    });

    // const context = await browser.newContext();
    // const app = await context.newPage();
    // await connect(app, page);
    // await app.getByTestId("sign-message-button").click();
    // await page.getByTestId('request-approve-button').click();
    // await expect(page.getByText("Signature approved")).toBeVisible();
    // await expect(app.getByText("Signing Succeeded")).toBeVisible();

    walletCDPSession.send("Network.emulateNetworkConditions", {
        offline: true,
        latency: 0,
        downloadThroughput: -1,
        uploadThroughput: -1,
    });

    const context2 = await browser.newContext();
    const app2 = await context2.newPage();
    await app2.goto("https://lab.reown.com/appkit/?name=wagmi");
    await app2.getByTestId("connect-button").click({ force: true });
    await app2.getByTestId("wallet-selector-walletconnect").click();
    const qr = app2.getByTestId("wui-qr-code");
    await expect(qr).toBeVisible();
    const uri = await qr.getAttribute("uri");

    const pairingUri = page.getByTestId("input-pairing-uri").locator('input');
    await expect(pairingUri).toBeVisible();
    await pairingUri.fill(uri!);
    await page.getByTestId('pair-submit-button').click();

    await expect(page.getByText('Approve pairing')).toBeVisible();
    await expect(page.getByText("Pairing failed:")).toBeVisible({ timeout: 11000 });
    await expect(page.getByText('Approve pairing')).not.toBeVisible();

    walletCDPSession.send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 0,
        downloadThroughput: -1,
        uploadThroughput: -1,
    });

    await expect(pairingUri).toBeVisible();
    await pairingUri.fill(uri!);
    await page.getByTestId('pair-submit-button').click();

    await page.getByTestId('pairing-approve-button').click();
    await expect(page.getByText("Pairing approved")).toBeVisible();
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(1);

    await app2.getByTestId("sign-message-button").click();
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await expect(app2.getByText("Signing Succeeded")).toBeVisible();
});

test('disconnect from wallet - Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    expect(await app.getByTestId("w3m-caip-address").innerText()).toEqual("eip155:1:0x0000000000000000000000000000000000000000");
    const disconnectButton = page.getByRole("button", { name: "Disconnect" });
    await expect(disconnectButton).toBeVisible();
    await expect(disconnectButton).toHaveCount(1);
    await disconnectButton.click();
    await expect(disconnectButton).toHaveCount(0);
    await expect(app.getByTestId("w3m-caip-address")).toHaveText("-");
});

test('disconnect from app - Rust wallet to JS app', async ({ browser, page, baseURL }) => {
    await page.goto(baseURL!);
    const context = await browser.newContext();
    const app = await context.newPage();
    await connectJsApp(app, page);
    expect(await app.getByTestId("w3m-caip-address").innerText()).toEqual("eip155:1:0x0000000000000000000000000000000000000000");
    await app.getByTestId("disconnect-hook-button").click({ force: true });
    await expect(app.getByTestId("w3m-caip-address")).toHaveText("-");
});
