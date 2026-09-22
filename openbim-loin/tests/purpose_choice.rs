//! Issue #5: repeating-choice setter semantics.
use openbim_loin::{
    dt::{Guid, MultiLanguageText},
    Purpose, PurposeItem,
};

fn purpose() -> Purpose {
    let guid: Guid = "10000000-0000-0000-0000-000000000000".parse().unwrap();
    let name = MultiLanguageText::new("en", "P").unwrap();
    let mut p = Purpose::new(guid, name);
    p.add_item(PurposeItem::Language("en".parse().unwrap()));
    p.add_item(PurposeItem::Language("de".parse().unwrap()));
    p
}

/// Issue #5: a singular setter must not delete schema-valid duplicates.
#[test]
fn set_language_preserves_repeated_branches() {
    let mut p = purpose();
    p.set_language(Some("fr".parse().unwrap())).unwrap();
    let langs: Vec<String> = p
        .languages()
        .iter()
        .map(|l| l.as_str().to_owned())
        .collect();
    assert_eq!(langs, vec!["fr".to_owned(), "de".to_owned()]);
}

/// Issue #5: set_all_* is the explicit opt-in to collapsing duplicates.
#[test]
fn set_all_languages_collapses_and_reports_removals() {
    let mut p = purpose();
    let removed = p.set_all_languages(Some("fr".parse().unwrap())).unwrap();
    assert_eq!(removed, 1, "one duplicate branch removed");
    let langs: Vec<String> = p
        .languages()
        .iter()
        .map(|l| l.as_str().to_owned())
        .collect();
    assert_eq!(langs, vec!["fr".to_owned()]);
}
