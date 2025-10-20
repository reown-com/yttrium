import { test, expect, type Page } from '@playwright/test';

test('connect Rust app to JS wallet', async ({ browser, page: app, context, baseURL }) => {
    await app.goto(baseURL!);
    await app.getByTestId("connect-button").click();
    await expect(app.getByTestId("pairing-uri")).toBeVisible();
    await expect(app.getByTestId("pairing-uri")).toHaveAttribute("data-pairing-uri");
    const uri = await app.getByTestId("pairing-uri").getAttribute("data-pairing-uri");

    const wallet = await context.newPage();
    await wallet.goto("https://react-wallet.walletconnect.com/walletconnect");
    await expect(wallet.getByTestId("uri-input")).toBeVisible();
    await expect(wallet.getByTestId("uri-input")).toBeEditable();
    await wallet.getByTestId("uri-input").fill(uri!);
    await wallet.getByTestId("uri-connect-button").click();
    await expect(wallet.getByTestId("session-approve-button")).toBeVisible();
    await expect(wallet.getByTestId("session-info-verified")).toBeVisible();
    await wallet.getByTestId("session-approve-button").click();

    await expect(app.getByTestId("request-button")).toBeVisible();
    await app.getByTestId("request-button").click();
    // note: misleading test ID
    await expect(wallet.getByTestId("session-approve-button")).toBeVisible();
    await expect(wallet.getByTestId("session-info-verified")).toBeVisible();
    await wallet.getByTestId("session-approve-button").click();
    await expect(app.getByText("Session request result")).toBeVisible();
});
