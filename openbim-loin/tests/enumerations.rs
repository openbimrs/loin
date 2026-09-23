//! Enumeration lexical values: one source for model, validator and I/O.
//!
//! The expected lists restate the ISO 7817-3 XSD `xs:enumeration` facets on
//! purpose. They are the independent oracle.
//! A model enum that drifts from the schema must fail here, not silently
//! change what validation accepts.

use openbim_loin::{Appearance, CoordinateReferenceSystemKind, Dimensionality};
use openbim_loin::{Connections, Features, InsideGeometry, Openings};
use openbim_loin::{OperatingAndClearanceZones, ParametricBehaviour};
use openbim_loin::{RelativeOrAbsolute, ShapeAssembly, ShapeRepresentation};
use std::str::FromStr;

/// Asserts VALUES matches the XSD and every value round-trips exactly.
macro_rules! check {
    ($ty:ty, [$($xml:literal),+ $(,)?]) => {{
        let expected: &[&str] = &[$($xml),+];
        assert_eq!(<$ty>::VALUES, expected, stringify!($ty));
        for value in expected {
            let parsed = <$ty>::from_str(value).expect(value);
            assert_eq!(parsed.as_str(), *value);
            assert_eq!(parsed.to_string(), *value);
        }
    }};
}

#[test]
fn enumerations_match_the_xsd_and_round_trip() {
    check!(
        ShapeAssembly,
        [
            "NotRequired",
            "SingleObjectSingularShape",
            "SingleObjectMultipleShapes",
            "MultipleObjects"
        ]
    );
    check!(
        ShapeRepresentation,
        [
            "NotRequired",
            "SingleBoundingPrimitive",
            "OuterShellAsSingularShape",
            "OuterShellAsSeparateShapes"
        ]
    );
    check!(
        InsideGeometry,
        [
            "NotRequired",
            "NoInsideGeometry",
            "InsideGeometryAsPartOfShape",
            "SeparateShapes"
        ]
    );
    check!(
        Connections,
        [
            "NotRequired",
            "NoConnections",
            "ConnectionsAsPartOfShape",
            "SeparateShapes"
        ]
    );
    check!(
        Openings,
        [
            "NotRequired",
            "NoOpenings",
            "OpeningsAsPartOfShape",
            "SeparateShapes"
        ]
    );
    check!(
        OperatingAndClearanceZones,
        [
            "NotRequired",
            "NoZones",
            "ZonesAsPartOfShape",
            "SeparateShapes"
        ]
    );
    check!(
        Features,
        [
            "NotRequired",
            "NoFeatures",
            "FeaturesAsPartOfShape",
            "SeparateShapes"
        ]
    );
    check!(Dimensionality, ["NotRequired", "0D", "1D", "2D", "3D"]);
    check!(
        Appearance,
        [
            "NotRequired",
            "NoAppearanceInformation",
            "SymbolicByMapping",
            "SingularMaterial",
            "MultipleMaterials",
            "ConceptualAppearance",
            "RealisticAppearance"
        ]
    );
    check!(ParametricBehaviour, ["NotRequested", "Requested"]);
    check!(RelativeOrAbsolute, ["NotDefined", "Absolute", "Relative"]);
    check!(
        CoordinateReferenceSystemKind,
        [
            "NotRequired",
            "ProjectedCRS",
            "EngineeringCRS",
            "GeographicCRS"
        ]
    );
}

/// Lexical matching is exact: XSD enumerations are case-sensitive and the
/// validator compares the collapsed text as-is.
#[test]
fn near_miss_values_are_rejected_with_context() {
    let error = Dimensionality::from_str("ZeroD").expect_err("Rust name");
    assert_eq!(error.enumeration(), "Dimensionality");
    assert_eq!(error.value(), "ZeroD");
    assert!(CoordinateReferenceSystemKind::from_str("ProjectedCrs").is_err());
    assert!(Appearance::from_str("notrequired").is_err());
}
