//! Differential oracle for issue #4: the implementation must agree with the
//! ISO 7817-3 `EMailAddressType` facet on every generated candidate.
use openbim_loin::EmailAddress;
use std::str::FromStr;

/// Faithful interpreter for the literal facet `[^@]+@[^\.]+\..+`, anchored.
fn xsd_pattern(value: &str) -> bool {
    let chars: Vec<char> = value.chars().collect();
    // [^@]+ : one or more non-@, then the first @.
    let Some(at) = chars.iter().position(|c| *c == '@') else {
        return false;
    };
    if at == 0 {
        return false;
    }
    let rest = &chars[at + 1..];
    // `[^\.]+` must match at least one NON-DOT character immediately after
    // the `@`, so the qualifying dot is the FIRST dot and it cannot sit at
    // index 0. `.+` then needs at least one character after it.
    let Some(dot) = rest.iter().position(|c| *c == '.') else {
        return false;
    };
    dot > 0 && dot + 1 < rest.len()
}

#[test]
fn implementation_matches_the_xsd_facet_exhaustively() {
    let alphabet = ['a', '@', '.', 'b'];
    let mut checked = 0_usize;
    let mut divergences: Vec<String> = Vec::new();

    for length in 1..=6_usize {
        let mut indices = vec![0_usize; length];
        loop {
            let candidate: String = indices.iter().map(|i| alphabet[*i]).collect();
            let expected = xsd_pattern(&candidate);
            let actual = EmailAddress::from_str(&candidate).is_ok();
            if expected != actual {
                divergences.push(candidate.clone());
            }
            checked += 1;

            let mut position = length;
            loop {
                if position == 0 {
                    break;
                }
                position -= 1;
                indices[position] += 1;
                if indices[position] < alphabet.len() {
                    break;
                }
                indices[position] = 0;
                if position == 0 {
                    break;
                }
            }
            if indices.iter().all(|i| *i == 0) {
                break;
            }
        }
    }

    assert!(checked > 5000, "corpus too small: {checked}");
    assert!(
        divergences.is_empty(),
        "{} divergences from the XSD facet, e.g. {:?}",
        divergences.len(),
        &divergences[..divergences.len().min(12)]
    );
}

/// Issue #4's concrete report: a leading `@` must be rejected.
#[test]
fn leading_at_is_rejected() {
    for bad in ["@a@b.c", "@b.c", "@@b.c"] {
        assert!(
            EmailAddress::from_str(bad).is_err(),
            "{bad:?} must be rejected"
        );
    }
}

/// The schema permits later `@` characters, so these stay ACCEPTED. Rejecting
/// them would deviate from the published facet.
#[test]
fn additional_at_after_the_first_is_schema_valid() {
    for good in ["a@@b.c", "a@b@c.de", "a@@@b.cd"] {
        assert!(
            EmailAddress::from_str(good).is_ok(),
            "{good:?} matches the XSD facet and must be accepted"
        );
    }
}

#[test]
fn ordinary_addresses_round_trip() {
    let address = EmailAddress::from_str("a@b.c").expect("valid");
    assert_eq!(address.to_string(), "a@b.c");
}
