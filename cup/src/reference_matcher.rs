use regex::Regex;
use std::hash::Hash;

/// When customizing image configuration, there may be a need to operate on a batch of images. The ReferenceMatcher describes which references the configuration should apply to.
pub enum ReferenceMatcher {
    /// The reference is exactly equal to the string provided
    ExactMatch(String),
    /// The reference starts with the string provided, useful for configuring based on a registry or repository
    PrefixMatch(String),
    /// The reference ends with the string provided, useful for configuring based on a tag
    SuffixMatch(String),
    /// The reference contains the string provided
    Contains(String),
    /// For more complicated matching logic a regular expression can be used.
    Custom(Regex),
}

impl PartialEq for ReferenceMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReferenceMatcher::Custom(a), ReferenceMatcher::Custom(b)) => a.as_str() == b.as_str(),
            (a, b) => a.eq(b),
        }
    }
}

impl Eq for ReferenceMatcher {}

impl Hash for ReferenceMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Custom(r) => r.as_str().hash(state),
            o => o.hash(state),
        }
    }
}

impl ReferenceMatcher {}
