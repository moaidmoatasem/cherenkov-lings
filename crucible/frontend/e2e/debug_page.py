from playwright.sync_api import sync_playwright
import time

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": 1440, "height": 900})
    page.goto("http://127.0.0.1:8180", wait_until="networkidle", timeout=30000)
    time.sleep(3)
    print("Title:", page.title())
    print("URL:", page.url)
    print("Body text:", page.evaluate("() => document.body.innerText")[:500])
    print("Has home-page:", page.evaluate("() => !!document.querySelector('[data-testid=home-page]')"))
    print("Has drills-grid:", page.evaluate("() => !!document.querySelector('.drills-grid')"))
    page.screenshot(path="C:/Users/moaid/Documents/antigravity/wonderful-raman/crucible/test-results/debug_screenshot.png", full_page=True)
    browser.close()
