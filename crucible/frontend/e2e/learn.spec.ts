import { test, expect } from '@playwright/test';

test.describe('Learn environment — read, watch, practice, build', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/learn');
    await expect(page.locator('.learn-root')).toBeVisible();
  });

  test('shell loads with header, sidebar and a11y toggles', async ({ page }) => {
    // The greeting and date come from the clock, so assert the shape rather
    // than a literal that was true only when it was written.
    await expect(page.locator('.l-h1')).toHaveText(/Good (morning|afternoon|evening), Moaid/);
    await expect(page.locator('.l-crumb')).toHaveText(
      /^(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday), \w+ \d+$/
    );
    // The note reports the record, once /api/progress has answered.
    await expect(page.locator('.l-header-note')).toHaveText(/^(\d+ of \d+ modules built)?$/);
    await expect(page.getByRole('navigation', { name: 'Sections' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Today' })).toHaveAttribute('aria-current', 'page');
    await expect(page.getByRole('button', { name: 'Cherenkov — back to the sandbox' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Bigger text' })).toHaveAttribute('aria-pressed', 'false');
    await expect(page.getByRole('button', { name: 'Easier-reading typeface' })).toHaveAttribute('aria-pressed', 'false');
    await expect(page.locator('.learn-root')).toHaveAttribute('data-type', 'md');
  });

  test('a11y toggles flip data attributes', async ({ page }) => {
    const bigger = page.getByRole('button', { name: 'Bigger text' });
    const easy = page.getByRole('button', { name: 'Easier-reading typeface' });
    await bigger.click();
    await expect(bigger).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.learn-root')).toHaveAttribute('data-type', 'lg');
    await bigger.click();
    await expect(page.locator('.learn-root')).toHaveAttribute('data-type', 'md');
    await easy.click();
    await expect(easy).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.learn-root')).toHaveAttribute('data-dys', 'on');
    await easy.click();
    await expect(page.locator('.learn-root')).toHaveAttribute('data-dys', 'off');
  });

  test('navigates all six screens via sidebar', async ({ page }) => {
    const nav = [
      { label: 'Today', heading: /Good (morning|afternoon|evening), Moaid/ },
      { label: 'This module', heading: 'Waiting without sleeping' },
      { label: 'Browser lab', heading: 'The lab' },
      { label: 'Device lab', heading: 'The device lab' },
      // Counts come from GET /api/curriculum, so assert the shape, not a literal.
      { label: 'All modules', heading: /\d+ tracks, \d+ modules/ },
      { label: 'My record', heading: 'What you can prove' },
    ];
    for (const { label, heading } of nav) {
      await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: label }).click();
      await expect(page.locator('.l-h1')).toContainText(heading);
      await expect(page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: label })).toHaveAttribute('aria-current', 'page');
    }
  });

  test('Today screen shows only figures the record can support', async ({ page }) => {
    // The guided module is labelled as a walkthrough, not as the learner's own
    // half-finished progress, and its steps carry no completion ticks.
    await expect(page.getByText('How a module works')).toBeVisible();
    await expect(page.getByText('Continue where you stopped')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: 'Waiting without sleeping' })).toBeVisible();
    await expect(page.locator('.l-continue-lede')).toContainText('A walkthrough of one module');
    await expect(page.locator('.l-loop-row[data-state="done"]')).toHaveCount(0);
    await expect(page.locator('.l-loop-label').filter({ hasText: 'Read the failure' })).toBeVisible();
    await expect(page.locator('.l-loop-label').filter({ hasText: 'Build it in the lab' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Open the lab and build it' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Re-read the page first' })).toBeVisible();

    // "Next up" replaces the invented 14:00/18:30/20:00 schedule. There is no
    // scheduler behind the product, and its Reschedule button did nothing.
    await expect(page.getByText('Next up')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Reschedule' })).toHaveCount(0);
    await expect(page.getByText('14:00')).toHaveCount(0);

    // The streak strip is exactly as long as the streak.
    await expect(page.getByText('Kept it up')).toBeVisible();
    const streak = await page.locator('.l-next-session-when').innerText();
    const days = Number.parseInt(streak, 10);
    await expect(page.locator('.l-dot')).toHaveCount(Math.max(days, 1));

    // Points are the learner's real total, not a weekly target nobody set.
    await expect(page.getByText('Points earned')).toBeVisible();
    await expect(page.getByText('of 900 points')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Start recall' })).toHaveCount(0);
  });

  test('Module screen read step renders article and diff', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'This module' }).click();
    await page.getByRole('tab', { name: /Read/ }).click();
    await expect(page.getByRole('tab', { name: /Read/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByText('Why a sleep is never a wait')).toBeVisible();
    await expect(page.getByText('Reading · 6 minutes')).toBeVisible();
    await expect(page.getByText('saved for offline')).toBeVisible();
    await expect(page.getByText('the app will be ready in one second')).toBeVisible();
    await expect(page.getByText('Assert on state, never on time.')).toBeVisible();
    await expect(page.locator('.l-diff-line[data-kind="removed"]')).toContainText('await page.waitForTimeout(1000);');
    await expect(page.locator('.l-diff-line[data-kind="added"]')).toContainText('await expect(results).toBeVisible();');
    await expect(page.getByText('In this page')).toBeVisible();
    await expect(page.getByRole('button', { name: 'The guess you make' })).toBeVisible();
    await expect(page.getByText('4 modules, and 2 recall questions')).toBeVisible();
    await expect(page.getByRole('button', { name: /Next · watch the trace/ })).toBeVisible();
    await page.getByRole('button', { name: /Next · watch the trace/ }).click();
    await expect(page.getByRole('tab', { name: /Watch/ })).toHaveAttribute('aria-selected', 'true');
  });

  test('Module screen watch step shows player and chapters', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'This module' }).click();
    await page.getByRole('tab', { name: /Watch/ }).click();
    await expect(page.getByLabel('Play the module video')).toBeVisible();
    await expect(page.getByText('Watch the sleep run out, in a real trace')).toBeVisible();
    await expect(page.getByText('3:12 / 9:24')).toBeVisible();
    await expect(page.getByText('Short on time')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Play the 90-second cut' })).toBeVisible();
    await expect(page.getByText('Chapters')).toBeVisible();
    await expect(page.getByText('The failing run')).toBeVisible();
    await expect(page.getByText('The wait gives up early')).toBeVisible();
    await expect(page.getByText('Note at 3:12')).toBeVisible();
    await expect(page.getByText('the response came back after the wait had already failed')).toBeVisible();
  });

  test('Module screen practice step shows question and answers', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'This module' }).click();
    await expect(page.getByRole('tab', { name: /Practice/ })).toHaveAttribute('aria-selected', 'true');
    // The screen states plainly that this is one fixed illustration, not the
    // learner's own quiz progress -- and there is no real 5-question sequence
    // behind it, so the position claim is gone too.
    await expect(page.locator('.l-meta').filter({ hasText: 'A worked example.' })).toBeVisible();
    await expect(page.getByText('A worked question · nothing is graded')).toBeVisible();
    await expect(page.getByText(/Question \d+ of \d+/)).toHaveCount(0);
    await expect(page.getByText('This passes on your laptop and fails one CI run in five. What actually fixes it?')).toBeVisible();
    await expect(page.locator('.l-snippet-line[data-kind="bad"]')).toContainText('waitForTimeout');
    await expect(page.getByRole('button', { name: /Raise the wait to three seconds/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Assert that the results are visible/ })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByText("That's the one")).toBeVisible();
    await expect(page.getByRole('button', { name: 'Now do it in the lab' })).toBeVisible();
    await expect(page.getByText('The five')).toBeVisible();
    await expect(page.getByText('Sleep against assertion')).toBeVisible();
    await expect(page.getByText('No penalty.')).toBeVisible();
    await page.getByRole('button', { name: 'Now do it in the lab' }).click();
    await expect(page.locator('.l-h1')).toContainText('The lab');
  });

  test('Browser lab shows code, preview and passing verdict', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'Browser lab' }).click();
    await expect(page.locator('.l-h1')).toContainText('The lab');
    await expect(page.getByText('Make the search test hold up five times in a row')).toBeVisible();
    await expect(page.getByRole('button', { name: 'passing run' })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.l-panel-file')).toContainText('search.spec.ts');
    // The dead "Run 5x" control is gone -- nothing on this screen executes.
    await expect(page.locator('.l-panel-run')).toHaveCount(0);
    await expect(page.locator('.l-panel-status')).toContainText('example run');
    await expect(page.locator('.l-lab-intro')).toContainText('A worked example.');
    await expect(page.locator('.l-lab-intro')).toContainText('cherenkov-lings watch');
    await expect(page.locator('.l-code-row').first()).toBeVisible();
    await expect(page.locator('.l-code-text', { hasText: "getByRole('searchbox'" }).first()).toBeVisible();
    await expect(page.locator('.l-preview-url')).toContainText('localhost:8080/search');
    await expect(page.locator('.l-preview-lat')).toContainText('+200 ms');
    await expect(page.locator('.l-preview-note')).toContainText('Static preview');
    await expect(page.locator('.l-preview-note').getByRole('link', { name: 'live /search' })).toBeVisible();
    await expect(page.locator('.l-annotation').first()).toContainText("getByRole('searchbox')");
    await expect(page.getByText('3 results visible')).toBeVisible();
    await expect(page.locator('.l-result-row')).toHaveCount(3);
    await expect(page.locator('.l-iter-label')).toContainText('run 4 of 5');
    await expect(page.locator('.l-iter-dot')).toHaveCount(5);
    await expect(page.getByText("Five runs, five passes. That's the skill.")).toBeVisible();
    await expect(page.getByText('+180 points')).toBeVisible();
    await expect(page.getByText('Survived five runs')).toBeVisible();
    await expect(page.getByText('Next module · Locators')).toBeVisible();
    await expect(page.getByText('You removed one sleep and added two assertions.')).toBeVisible();
  });

  test('Browser lab failing run shows failures and hints', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'Browser lab' }).click();
    await page.getByRole('button', { name: 'failing run' }).click();
    await expect(page.getByRole('button', { name: 'failing run' })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByText('Two of five runs failed — start at the top')).toBeVisible();
    await expect(page.locator('.l-failure')).toHaveCount(3);
    await expect(page.getByText('Your wait is shorter than the response')).toBeVisible();
    await expect(page.getByText('fix first')).toBeVisible();
    await expect(page.getByText('Why it broke')).toBeVisible();
    await expect(page.getByText('Run 3 got its response 240 ms after')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Show me the line to change' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Run it again, 5×' })).toBeVisible();
    await expect(page.getByText("Hints don't cost points.")).toBeVisible();
  });

  test('Device lab shows yaml, conditions and handset preview', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'Device lab' }).click();
    await expect(page.locator('.l-h1')).toContainText('The device lab');
    await expect(page.getByText("Face ID isn't available after a restart.")).toBeVisible();
    await expect(page.locator('.l-panel-file')).toContainText('biometric_fallback.yaml');
    await expect(page.locator('.l-panel-run')).toHaveCount(0);
    await expect(page.getByText('Pixel 7 · Android 14')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Play on device' })).toHaveCount(0);
    await expect(page.locator('.l-code-row')).toContainText(['appId: dev.cherenkov.bank']);
    await expect(page.getByText('Make it harder')).toBeVisible();
    await expect(page.locator('.l-cond')).toHaveCount(4);
    await expect(page.getByRole('button', { name: /Network/ })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByRole('button', { name: /Process/ })).toHaveAttribute('aria-pressed', 'true');
    const network = page.getByRole('button', { name: /Network/ });
    await network.click();
    await expect(network).toHaveAttribute('aria-pressed', 'false');
    await network.click();
    await expect(network).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.l-device-preview-note')).toContainText('Static preview');
    await expect(page.locator('.l-device-preview-note').getByRole('link', { name: 'live sandbox' })).toBeVisible();
    await expect(page.locator('.l-screen-title')).toContainText('Unlock to continue');
    await expect(page.getByText('Biometrics unavailable after restart')).toBeVisible();
    await expect(page.locator('.l-screen-cta')).toContainText('Use passcode instead');
    await expect(page.locator('.l-screen-tag')).toContainText('tapping now');
    await expect(page.locator('.l-label').filter({ hasText: 'The flow' })).toBeVisible();
    await expect(page.locator('.l-flow-step')).toHaveCount(6);
    await expect(page.getByText('Handle the refusal')).toBeVisible();
  });

  test('All modules shows search, filters and tracks', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'All modules' }).click();
    await expect(page.locator('.l-h1')).toContainText(/\d+ tracks, \d+ modules/);
    const search = page.getByPlaceholder('Search modules, notes, error messages…');
    await expect(search).toBeVisible();
    await expect(search).toHaveAttribute('aria-label', 'Search modules');
    await expect(page.getByRole('button', { name: 'Everything' })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByRole('button', { name: 'Not started' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Has a video' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Under 20 min' })).toBeVisible();
    await expect(page.getByText('Foundations')).toBeVisible();
    await expect(page.getByText('Modern Web Automation')).toBeVisible();
    await expect(page.locator('.l-mod-row')).not.toHaveCount(0);
    await search.fill('waiting');
    await expect(page.getByText('Waiting without sleeping')).toBeVisible();
    await search.fill('zzzz-no-match-xyz');
    await expect(page.getByText('Nothing matches that yet.')).toBeVisible();
    await search.fill('');
    await page.getByRole('button', { name: 'Has a video' }).click();
    await expect(page.getByRole('button', { name: 'Has a video' })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.l-mod-row')).not.toHaveCount(0);
    await page.getByRole('button', { name: 'Everything' }).click();
    await expect(page.getByRole('button', { name: 'Everything' })).toHaveAttribute('aria-pressed', 'true');
  });

  test('My record shows only earned evidence', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'My record' }).click();
    await expect(page.locator('.l-h1')).toContainText('What you can prove');

    // Every KPI is measured. "Kept sessions" and "Time spent" are gone: the
    // platform records neither, and they used to sit beside a real count they
    // contradicted.
    await expect(page.locator('.l-kpi')).toHaveCount(4);
    await expect(page.getByText('Modules built')).toBeVisible();
    await expect(page.getByText('Tracks started')).toBeVisible();
    await expect(page.getByText('Day streak')).toBeVisible();
    await expect(page.getByText('Kept sessions')).toHaveCount(0);
    await expect(page.getByText('Time spent')).toHaveCount(0);

    // Modules built agrees with the per-track rows. Polled, because the count
    // and the track list arrive from two independent requests.
    await expect
      .poll(async () => {
        const built = Number.parseInt(
          await page
            .locator('.l-kpi')
            .filter({ hasText: 'Modules built' })
            .locator('.l-kpi-value')
            .innerText(),
          10
        );
        const perTrack = await page.locator('.l-skill-stage').allInnerTexts();
        const summed = perTrack.reduce((n, t) => n + (Number.parseInt(t, 10) || 0), 0);
        return summed === built;
      })
      .toBe(true);

    // The certificate reflects a real track, with no invented delivery date.
    await expect(page.locator('.l-cert-pip')).not.toHaveCount(0);
    await expect(page.getByText('around Sep 6')).toHaveCount(0);

    // Badges are the ones the engine actually unlocked, and the fabricated
    // public profile URL is gone.
    await expect(page.getByText('Already yours')).toBeVisible();
    await expect(page.getByText('cherenkov.dev/moaid')).toHaveCount(0);
  });

  test('brand wordmark exits back to sandbox', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Cherenkov — back to the sandbox' })).toBeVisible();
    await page.getByRole('button', { name: 'Cherenkov — back to the sandbox' }).click();
    await expect(page).toHaveURL('/sandbox');
    // /sandbox renders the sandbox inside the Learn shell, so the assertion is
    // the sandbox's own content rather than the standalone page's footer.
    await expect(page.locator('.l-h1')).toContainText('Micro-Crucible Sandbox');
    await expect(page.getByText('Drill 01: Hydration Timing Gap')).toBeVisible();
  });

  test('Navbar learn entry routes correctly', async ({ page }) => {
    // Started from a page that carries the Navbar: /sandbox now renders inside
    // the Learn shell, which has the sidebar rather than the sandbox header.
    await page.goto('/checkout');
    await expect(page.getByTestId('nav-learn')).toBeVisible();
    await page.getByTestId('nav-learn').click();
    await expect(page).toHaveURL(/\/learn/);
    await expect(page.locator('.l-h1')).toHaveText(/Good (morning|afternoon|evening), Moaid/);
    await expect(page.locator('.learn-root')).toBeVisible();
    await expect(page.locator('.site-footer')).toHaveCount(0);
  });

  test('root now serves latest Learn UI', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL('/');
    await expect(page.locator('.learn-root')).toBeVisible();
    await expect(page.locator('.l-h1')).toHaveText(/Good (morning|afternoon|evening), Moaid/);
    await expect(page.locator('.site-footer')).toHaveCount(0);
  });
  test('Navbar overview entry reaches the sandbox home page', async ({ page }) => {
    await page.goto('/checkout');
    await page.getByTestId('nav-sandbox').click();
    await expect(page).toHaveURL('/sandbox');
    await expect(page.getByText('Drill 01: Hydration Timing Gap')).toBeVisible();
  });

  test('an unknown route renders a not-found page, not the Learn app', async ({ page }) => {
    await page.goto('/no-such-page');
    await expect(page.getByTestId('not-found')).toBeVisible();
    await expect(page.locator('.learn-root')).toHaveCount(0);
    await page.getByRole('link', { name: 'Sandbox overview' }).click();
    await expect(page).toHaveURL('/sandbox');
  });

  test('the lab read link opens the module read step', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'Browser lab' }).click();
    await page.getByRole('link', { name: 'Read: auto-waiting, in depth' }).click();
    await expect(page.locator('.l-h1')).toContainText('Waiting without sleeping');
    await expect(page.getByRole('tab', { name: /^Read/ })).toHaveAttribute('aria-current', 'step');
  });

  test('the catalog lists every track the manifest declares', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'All modules' }).click();
    // A track with no curated copy still arrives, straight from lings.toml.
    // Waiting on it first also pins the moment the manifest has replaced the
    // curated seed the catalog paints with, so the count below is the settled
    // one rather than whichever render this read happened to catch.
    await expect(page.getByText('CI/CD Pipeline Engineering')).toBeVisible();
    const heading = await page.locator('.l-h1').textContent();
    const declared = Number(/(\d+) tracks/.exec(heading ?? '')?.[1]);
    expect(declared).toBeGreaterThan(4);
    await expect(page.locator('.l-track-name')).toHaveCount(declared);
  });
  test('a manifest drill opens its own theory and hints', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'All modules' }).click();
    await page.getByRole('button', { name: /Docker Socket Mount/ }).click();

    const drill = page.getByTestId('drill-screen');
    await expect(drill).toBeVisible();
    // Its own title, not the one hardcoded module every row used to open.
    await expect(page.locator('.l-h1')).toContainText('Docker Socket Mount');
    await expect(page.locator('.l-h1')).not.toContainText('Waiting without sleeping');
    // Real theory.md, rendered as prose rather than raw markdown.
    await expect(drill.locator('.l-md')).toBeVisible();
    await expect(drill.locator('.l-md h2, .l-md h3').first()).toBeVisible();
    await expect(drill.getByText('cherenkov-lings watch --track=devsecops-python')).toBeVisible();

    await drill.getByRole('button', { name: 'Hints' }).click();
    await expect(drill.locator('.l-md')).toBeVisible();

    await drill.getByRole('button', { name: '← All modules' }).click();
    await expect(page.locator('.l-h1')).toContainText(/tracks, \d+ modules/);
  });

  test('a curated module still opens the hand-written module screen', async ({ page }) => {
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('button', { name: 'All modules' }).click();
    await page.getByRole('button', { name: /Waiting without sleeping/ }).click();
    await expect(page.locator('.l-h1')).toContainText('Waiting without sleeping');
    await expect(page.getByTestId('drill-screen')).toHaveCount(0);
    await expect(page.getByRole('tablist', { name: 'Module steps' })).toBeVisible();
  });
});
