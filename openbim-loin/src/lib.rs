//! `openbim-loin` — ISO 7817-3 / EN 17412-3 Level of Information Need.
//!
//! # What this is
//!
//! A machine-readable statement of *how much* information is required about
//! which objects, for a given purpose, at a given milestone, between a given
//! pair of actors: geometric detail, alphanumeric properties, and
//! documentation.
//!
//! EN 17412-1 defines the concepts in prose; part 3 is the exchange format.
//! Only part 3 is implementable, and it is what this crate targets.
//!
//! # Relationship to ISO 23387
//!
//! The LOIN schema imports the ISO 23387 namespace for its property vocabulary.
//! This crate therefore uses and re-exports `openbim-dt` contracts for GUIDs,
//! multilingual text, references, concept inheritance, and embedded property,
//! quantity, group, document, template, unit, and dimension content.
//!
//! # Namespace versions
//!
//! Known documents use the `https://iso.org/2022/LOIN` and
//! `https://iso.org/2024/LOIN` draft namespaces. Namespace migration is therefore
//! a *first-class* concern here: reading accepts known historical namespaces,
//! and writing targets one explicitly rather than defaulting to whatever was
//! parsed.
//!
//! # Status
//!
//! The owned DT-backed domain boundary and a strict, bounded XML document codec
//! are implemented. The codec preserves semantically relevant XML syntax and
//! unknown content, supports explicit 2022/2024 namespace migration, and exposes
//! XSD-derived ISO 7817-3 structural and lexical diagnostics. Validation is
//! clause-level rather than a claim of complete XML Schema validation: imported
//! ISO 23387 complex-type internals remain owned by `openbim-dt` and outside the
//! validator's complete coverage.
//!
//! The ISO XSD is **not vendored**. Both the ISO/CEN originals and the public
//! committee drafts are unlicensed for redistribution, and the schema is a
//! moving target; it is referenced out of tree instead.

#![forbid(unsafe_code)]

mod authoring;
mod document;
mod model;
mod parser;
mod reader;
mod validation;

pub use authoring::AuthoringError;
pub use reader::{ReadError, ReadErrorKind};

/// Why ISO 23387-owned complex content cannot be written by this crate.
///
/// Exposed so consumers can match on the reason rather than on message text.
pub const DT_UNWRITABLE_REASON: &str = "openbim-dt 0.2 exposes no serializer for its owned types and dt::Element cannot be built downstream";
pub use document::{
    LoinDocument, MigrationError, MigrationReport, NamespaceVersion, OutputNamespace, WriteError,
    XmlAttribute, XmlDeclaration, XmlElement, XmlNode,
};
pub use model::*;
/// Exact ISO 23387 contract version consumed by this LOIN release.
pub use openbim_dt as dt;
pub use parser::{ParseError, ParseErrorKind, ParseOptions};
pub use validation::{Diagnostic, DiagnosticCode, Severity};

/// The namespace declared by the ISO 7817-3 draft schema (2024).
pub const NAMESPACE_2024: &str = "https://iso.org/2024/LOIN";

/// The namespace declared by the earlier EN 17412-3 committee draft (2022).
///
/// Retained because documents using it exist. Reading accepts it and writing
/// can preserve or target it only through an explicit [`OutputNamespace`] policy.
pub const NAMESPACE_2022: &str = "https://iso.org/2022/LOIN";

/// Namespaces this crate recognises as LOIN, newest first.
///
/// Ordered so that a reader trying candidates in sequence prefers the current
/// one, and so that adding the final published namespace is a one-line change
/// at the front of the list.
pub const KNOWN_NAMESPACES: &[&str] = &[NAMESPACE_2024, NAMESPACE_2022];

/// Whether a namespace URI is a LOIN namespace this crate knows.
///
/// ```
/// use openbim_loin::{is_known_namespace, NAMESPACE_2024};
/// assert!(is_known_namespace(NAMESPACE_2024));
/// assert!(!is_known_namespace("https://example.invalid/LOIN"));
/// ```
#[must_use]
pub fn is_known_namespace(ns: &str) -> bool {
    KNOWN_NAMESPACES.contains(&ns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_draft_namespaces_are_recognised() {
        assert!(is_known_namespace(NAMESPACE_2024));
        assert!(is_known_namespace(NAMESPACE_2022));
    }

    #[test]
    fn unknown_namespace_is_rejected() {
        assert!(!is_known_namespace(""));
        assert!(!is_known_namespace("https://iso.org/2099/LOIN"));
    }

    #[test]
    fn current_namespace_is_preferred_first() {
        assert_eq!(KNOWN_NAMESPACES.first(), Some(&NAMESPACE_2024));
    }
}
