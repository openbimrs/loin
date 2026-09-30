//! Issue #8: `Specification` and `Purpose` names can be replaced and read back;
//! issue #7: `Diagnostic` is a `std::error::Error`.

use std::str::FromStr;

use openbim_dt::{Guid, MultiLanguageText};
use openbim_loin::*;

fn guid(value: &str) -> Guid {
    Guid::from_str(value).unwrap()
}
fn text(value: &str) -> MultiLanguageText {
    MultiLanguageText::new("en", value).unwrap()
}

#[test]
fn purpose_set_name_replaces_in_place_and_reads_back() {
    let mut purpose = Purpose::new(guid("10000000-0000-0000-0000-000000000000"), text("Old"));
    purpose.add_description(text("Described"));
    purpose.set_name(Some(text("New"))).unwrap();
    assert_eq!(purpose.name(), Some(&text("New")));
    assert_eq!(purpose.names().len(), 1);
    assert!(matches!(purpose.items()[0], PurposeItem::Name(_)));

    purpose.set_name(None).unwrap();
    assert_eq!(purpose.name(), None);
    assert_eq!(purpose.items().len(), 1);

    purpose.set_name(Some(text("Again"))).unwrap();
    assert_eq!(purpose.name(), Some(&text("Again")));
}

#[test]
fn purpose_set_name_refuses_to_empty_the_choice() {
    let mut purpose = Purpose::new(guid("10000000-0000-0000-0000-000000000000"), text("Only"));
    assert_eq!(purpose.set_name(None), Err(EmptyPurpose));
    assert_eq!(purpose.name(), Some(&text("Only")));
}

#[test]
fn specification_set_name_reads_back() {
    let purpose = Purpose::new(guid("10000000-0000-0000-0000-000000000000"), text("P"));
    let prerequisites = Prerequisites::new(
        guid("20000000-0000-0000-0000-000000000000"),
        purpose,
        InformationDeliveryMilestone::new(guid("30000000-0000-0000-0000-000000000000"), text("M")),
        Actor::new(guid("40000000-0000-0000-0000-000000000000"), text("A")),
        Actor::new(guid("50000000-0000-0000-0000-000000000000"), text("B")),
    );
    let mut specification = Specification::new(
        guid("60000000-0000-0000-0000-000000000000"),
        "Old",
        prerequisites,
    );
    specification.set_name("New");
    assert_eq!(specification.name(), "New");
}

#[test]
fn diagnostic_is_a_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<Diagnostic>();
}
