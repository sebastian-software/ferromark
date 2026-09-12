import json,re
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
root = Path('/Users/sebastian/Workspace/.codex/15d0/ferromark')
data = json.loads((root/'homepage/app/data/native-benchmarks.json').read_text())
with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={'width':1440,'height':1000})
    errors=[]
    page.on('pageerror',lambda error:errors.append(str(error)))
    page.goto('http://127.0.0.1:4991/ferromark/',wait_until='networkidle')
    select=page.get_by_label('Workload',exact=True)
    for table in data['tables']:
        select.select_option(table['case'])
        region=page.get_by_role('region',name='Native engine timings: '+table['label'],exact=True)
        expect(region.locator('tbody tr')).to_have_count(len(table['rows']))
        expect(region.get_by_role('link',name='Ferromark',exact=True)).to_have_count(1)
        for value in table['rows']:
            row=region.locator('tbody tr').filter(has=page.get_by_role('link',name=value['label'],exact=True))
            expect(row.locator('td').nth(0)).to_have_text(value['latency'])
            expect(row.locator('td').nth(1)).to_have_text(value['throughput'])
            expect(row.locator('td').nth(2)).to_have_text(value['relativeSpeed'])
            expect(row.locator('strong')).to_have_count(4 if value['winner'] else 0)
        for name in table['unmeasured']:
            expect(region.get_by_role('link',name=name,exact=True)).to_have_count(0)
    select.select_option('gfm_overlap/features')
    panel=page.locator('section').filter(has=page.get_by_role('heading',name='Compare Rust, Go, and .NET parser cores'))
    panel.screenshot(path='/private/tmp/ferromark-ox-implementation/native-desktop.png')
    page.set_viewport_size({'width':390,'height':844})
    panel.scroll_into_view_if_needed()
    assert page.evaluate('document.documentElement.scrollWidth <= window.innerWidth'), 'Mobile page overflow'
    page.screenshot(path='/private/tmp/ferromark-ox-implementation/native-mobile.png')
    page.goto('http://127.0.0.1:4991/ferromark/guide/benchmarks/',wait_until='networkidle')
    for table in data['tables']:
        region=page.get_by_role('region',name='Native engine timings: '+table['label'],exact=True)
        expect(region.locator('tbody tr')).to_have_count(len(table['rows']))
        expect(region.get_by_role('link',name='Ferromark',exact=True)).to_have_count(1)
    expect(page.get_by_role('region',name='Native comparison coverage').locator('tbody tr')).to_have_count(6)
    assert not errors, errors
    browser.close()
print('Browser checks passed: one Ferromark row per document, all measured times and relative speeds, winner emphasis, missing cases, mobile overflow, guide coverage, no runtime errors.')
