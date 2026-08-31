// Shared default values for the scrape request's `days`/`regions` filters —
// extracted here so the GUI (`src-tauri/src/scraper/commands.rs::run_scraper`)
// and the headless CLI (`cli/src/bin/scrape.rs`) apply the exact same defaults
// instead of duplicating the literals. Unlike `days`/`regions`, `classes`
// intentionally does NOT get a shared default: the GUI keeps its `"all"`
// sentinel (single global-ranking request), while the CLI always resolves to
// every seeded class individually (see `cli/src/bin/scrape.rs`).

/// Returns `days` unchanged if non-empty, otherwise the same 7-value default
/// the GUI has always used.
pub fn resolve_days(days: Vec<String>) -> Vec<String> {
    if days.is_empty() {
        vec!["20", "30", "60", "90", "180", "365", "ever"]
            .into_iter()
            .map(String::from)
            .collect()
    } else {
        days
    }
}

/// Returns `regions` unchanged if non-empty, otherwise the same 10-value
/// default the GUI has always used.
pub fn resolve_regions(regions: Vec<String>) -> Vec<String> {
    if regions.is_empty() {
        vec!["eu", "na", "ru", "jp", "kr", "tw", "sa", "sea", "asia", "mena"]
            .into_iter()
            .map(String::from)
            .collect()
    } else {
        regions
    }
}
