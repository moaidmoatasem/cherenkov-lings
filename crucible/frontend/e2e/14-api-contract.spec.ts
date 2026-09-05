import { test, expect } from '@playwright/test';

const API_BASE = 'http://localhost:8099';

test.describe('Backend API Contract Tests', () => {
  test('health endpoint returns ok', async ({ request }) => {
    const res = await request.get(`${API_BASE}/health`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data.status).toBe('ok');
    expect(data.service).toBe('micro-crucible-backend');
  });

  test('checkout get returns cart state', async ({ request }) => {
    const res = await request.get(`${API_BASE}/checkout`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('cart');
    expect(data).toHaveProperty('total');
  });

  test('checkout post creates order', async ({ request }) => {
    const res = await request.post(`${API_BASE}/checkout`, {
      data: { item_id: 'item-1', customer_name: 'QA', payment_method: 'credit_card' }
    });
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('order_id');
    expect(data.status).toBe('success');
  });

  test('balance returns account data', async ({ request }) => {
    const res = await request.get(`${API_BASE}/balance?account_id=ACC-001`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('balance');
    expect(data).toHaveProperty('account_id', 'ACC-001');
  });

  test('search returns matching results', async ({ request }) => {
    const res = await request.get(`${API_BASE}/search?q=playwright`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('results');
    expect(data.results.length).toBeGreaterThan(0);
  });

  test('products pagination works', async ({ request }) => {
    const res = await request.get(`${API_BASE}/products?page=1&per_page=5`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data.products.length).toBe(5);
    expect(data.page).toBe(1);
  });

  test('auth login returns token', async ({ request }) => {
    const res = await request.post(`${API_BASE}/auth/login`, {
      data: { username: 'sdet_student', password: 'secret' }
    });
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('access_token');
  });

  test('payment frame returns HTML', async ({ request }) => {
    const res = await request.get(`${API_BASE}/embed/payment-frame`);
    expect(res.status()).toBe(200);
    expect(res.headers()['content-type']).toContain('text/html');
  });

  test('rag endpoint returns grounded answer', async ({ request }) => {
    const res = await request.get(`${API_BASE}/api/rag?query=cherenkov`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('answer');
    expect(data.grounded).toBe(true);
  });

  test('llm endpoint returns stable intent', async ({ request }) => {
    const res = await request.get(`${API_BASE}/api/llm?prompt=transfer`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('intent', 'transfer_status_inquiry');
  });

  test('security user-lookup with SQLi returns timing warning', async ({ request }) => {
    const res = await request.get(`${API_BASE}/api/security/user-lookup?user_id=1%20OR%20SLEEP(1)`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('warning');
  });

  test('security fetch-url blocks SSRF', async ({ request }) => {
    const res = await request.post(`${API_BASE}/api/security/fetch-url`, {
      data: { url: 'http://169.254.169.254/latest/meta-data/' }
    });
    expect(res.status()).toBe(403);
  });

  test('pipeline validate checks YAML', async ({ request }) => {
    const res = await request.post(`${API_BASE}/api/pipeline/validate`, {
      data: { workflow_yaml: 'name: test\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok' }
    });
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('valid');
  });

  test('reports allure returns summary', async ({ request }) => {
    const res = await request.get(`${API_BASE}/api/reports/allure`);
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data).toHaveProperty('total_tests');
    expect(data).toHaveProperty('pass_percentage');
  });
});
