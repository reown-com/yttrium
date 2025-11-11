import test, { expect } from "@playwright/test";

test('self-connect Rust wallet to JS app', async ({ page, baseURL }) => {
    await page.goto(baseURL!);
    await expect(page.getByTestId("app-sessions").locator('li')).toHaveCount(0);
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(0);
    await page.getByTestId("connect-button").click();
    await expect(page.getByTestId("self-connect-button")).toBeVisible({ timeout: 10000 });
    await page.getByTestId("self-connect-button").click();
    await expect(page.getByText(`VerifyContext { origin: Some("${baseURL}"), validation: Valid, is_scam: false }`)).toBeVisible();
    await page.getByTestId("pairing-approve-button").click();
    await expect(page.getByTestId("app-sessions").locator('> li')).toHaveCount(1);
    await expect(page.getByTestId("wallet-sessions").locator('> li')).toHaveCount(1);
});

test('sessionUpdate standalone', async ({ page, baseURL }) => {
    await page.goto(baseURL!);
    await expect(page.getByTestId("app-sessions").locator('li')).toHaveCount(0);
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(0);
    await page.getByTestId("connect-button").click();
    await expect(page.getByTestId("self-connect-button")).toBeVisible({ timeout: 10000 });
    await page.getByTestId("self-connect-button").click();
    await expect(page.getByText(`VerifyContext { origin: Some("${baseURL}"), validation: Valid, is_scam: false }`)).toBeVisible();
    await page.getByTestId("pairing-approve-button").click();
    await expect(page.getByTestId("wallet-sessions").locator('li').first()).toBeVisible();
    await expect(page.getByTestId("app-sessions").locator('li').first()).toBeVisible();
    const walletSession = page.getByTestId("wallet-sessions").locator('li').first();
    await expect(walletSession.locator('li').first()).toBeVisible();
    const originalAccount = (await walletSession.locator('li').first().textContent())!.trim();
    await expect(walletSession.locator('li')).toHaveCount(1);
    const topic = await page.evaluate(() => {
        const state = localStorage.getItem('wc-wallet');
        if (!state) return null;
        const parsed = JSON.parse(state);
        return parsed.sessions?.[0]?.topic || null;
    });
    expect(topic).toBeTruthy();
    const newAddress = '0x3333333333333333333333333333333333333333';
    await page.evaluate(({ topic: t, address: addr }) => {
        const namespaces = {
            'eip155': {
                accounts: [`eip155:1:${addr}`],
                methods: ['personal_sign', 'eth_sendTransaction'],
                events: ['chainChanged', 'accountsChanged'],
                chains: ['eip155:1']
            }
        };
        return (window as any).sessionUpdate(t, namespaces);
    }, { topic, address: newAddress });
    const newAccountText = `eip155:1:${newAddress}`;
    await expect(page.getByTestId("wallet-sessions")).toContainText(newAccountText);
    await expect(walletSession.locator('li')).toHaveCount(1);
    await expect(walletSession.locator('li')).not.toContainText(originalAccount);
    await expect(walletSession.locator('li')).toContainText(newAccountText);
});

test('sessionUpdate with multiple chains', async ({ page, baseURL }) => {
    await page.goto(baseURL!);
    await expect(page.getByTestId("app-sessions").locator('li')).toHaveCount(0);
    await expect(page.getByTestId("wallet-sessions").locator('li')).toHaveCount(0);
    await page.getByTestId("connect-button").click();
    await expect(page.getByTestId("self-connect-button")).toBeVisible({ timeout: 10000 });
    await page.getByTestId("self-connect-button").click();
    await expect(page.getByText(`VerifyContext { origin: Some("${baseURL}"), validation: Valid, is_scam: false }`)).toBeVisible();
    await page.getByTestId("pairing-approve-button").click();
    await expect(page.getByTestId("wallet-sessions").locator('li').first()).toBeVisible();
    await expect(page.getByTestId("app-sessions").locator('li').first()).toBeVisible();
    const topic = await page.evaluate(() => {
        const state = localStorage.getItem('wc-wallet');
        if (!state) return null;
        const parsed = JSON.parse(state);
        return parsed.sessions?.[0]?.topic || null;
    });
    expect(topic).toBeTruthy();
    const address1 = '0x4444444444444444444444444444444444444444';
    const address2 = '0x5555555555555555555555555555555555555555';
    await page.evaluate(({ topic: t, addr1, addr2 }) => {
        const namespaces = {
            'eip155': {
                accounts: [`eip155:1:${addr1}`, `eip155:11155111:${addr2}`],
                methods: ['personal_sign', 'eth_sendTransaction'],
                events: ['chainChanged', 'accountsChanged'],
                chains: ['eip155:1', 'eip155:11155111']
            }
        };
        return (window as any).sessionUpdate(t, namespaces);
    }, { topic, addr1: address1, addr2: address2 });
    const account1Text = `eip155:1:${address1}`;
    const account2Text = `eip155:11155111:${address2}`;
    await expect(page.getByTestId("wallet-sessions")).toContainText(account1Text);
    await expect(page.getByTestId("wallet-sessions")).toContainText(account2Text);
    const appSession = page.getByTestId("app-sessions").locator('li').first();
    await expect(appSession.locator('li')).toHaveCount(2);
    const requestButtons = appSession.locator('li').locator('button[data-testid="request-button"]');
    await expect(requestButtons).toHaveCount(2);
    await requestButtons.nth(0).click();
    await expect(page.getByTestId('request-approve-button')).toBeVisible();
    await expect(page.getByText("Signature request")).toBeVisible();
    const dialog1 = page.getByText("Signature request").locator('..').locator('..');
    const dialogContent1 = await dialog1.locator('.thaw-dialog-content').textContent();
    expect(dialogContent1).toContain(`eip155:1`);
    expect(dialogContent1).toContain(address1);
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
    await requestButtons.nth(1).click();
    await expect(page.getByTestId('request-approve-button')).toBeVisible();
    await expect(page.getByText("Signature request")).toBeVisible();
    const dialog2 = page.getByText("Signature request").locator('..').locator('..');
    const dialogContent2 = await dialog2.locator('.thaw-dialog-content').textContent();
    expect(dialogContent2).toContain(`eip155:11155111`);
    expect(dialogContent2).toContain(address2);
    await page.getByTestId('request-approve-button').click();
    await expect(page.getByText("Signature approved")).toBeVisible();
});
