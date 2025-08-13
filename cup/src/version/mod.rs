use crate::version::date::DateVersion;
use crate::version::digest::DigestVersion;
use crate::version::extended::ExtendedVersion;
use crate::version::standard::StandardVersion;

pub mod date;
pub mod digest;
pub mod extended;
pub mod standard;

/// An enum used to refer to the version of an image independently of its versioning scheme. It strives to unify all common operations in one place so that there is no need to match on the type everywhere.
/// An alternative to this implementation would probably be having a `Version` trait and a lot of `Box` stuff going on which I'd rather avoid. For future readers: if you can think of a better way to handle this, I'm interested.
pub enum Version {
    /// Used when the versioning scheme cannot be determined and as the default. Should ideally be avoided, especially in user code. TODO: Check if this enum is actually useful in user code, maybe none of it should be used.
    Unknown,
    /// There isn't any official "standard" versioning scheme. It's just a sane default which basically describes the versioning scheme previous versions of Cup would handle. Refer to `StandardVersion` for more info.
    Standard(StandardVersion),
    /// Likewise, "extended" is my own creation for describing a more customizable versioning scheme. Refer to `ExtendedVersion` for more info.
    Extended(ExtendedVersion),
    /// The "Date" versioning schema is for images which are versioned based on date and or time and come with their own quirky rules to compare them. Refer to `DateVersion` for more info.
    Date(DateVersion),
    /// Version checking based on local and remote image digests
    Digest(DigestVersion)
}

pub enum VersionType {
    /// Tries to automatically infer the version type.
    Auto,
    Standard,
    Extended,
    Date,
    Digest
}

mod version_component {
    use std::{fmt::Display, num::ParseIntError, str::FromStr};

    /// A struct describing a version component. The objective is to store the _string length_ of it alongside the numeric value so it can be padded with the required amount of zeroes when converted back to string representation.
    #[derive(PartialEq, Debug)]
    pub struct VersionComponent {
        pub value: u32,
        pub length: u8, // An OCI image tag can only be up to 127 characters long so it's impossible for length to exceed a u8. See https://github.com/distribution/reference/blob/727f80d42224f6696b8e1ad16b06aadf2c6b833b/regexp.go#L68.
    }

    impl FromStr for VersionComponent {
        type Err = ParseIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                value: s.parse()?,
                length: s.len() as u8, // Cast is safe because `value.len()` is guaranteed to be much smaller than 255 characters. Refer to the comment on the `length` field of `VersionComponent`
            })
        }
    }

    impl Display for VersionComponent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:0>zeroes$}", self.value, zeroes = self.length as usize)
        }
    }

    impl PartialOrd for VersionComponent {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            if self.length == other.length {
                // We can't compare `2` with `01`. If you choose to zero-pad your numbers you'll have to do that everywhere, sorry.
                self.value.partial_cmp(&other.value)
            } else {
                None
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::str::FromStr;

        use super::VersionComponent;

        #[test]
        fn parse_from_string_slice() {
            let version_component = "0021";
            assert_eq!(
                VersionComponent::from_str(version_component).unwrap(),
                VersionComponent {
                    value: 21,
                    length: 4
                }
            );
        }

        #[test]
        fn stringify() {
            let version_component = VersionComponent {
                value: 21,
                length: 4,
            };
            assert_eq!(version_component.to_string(), "0021");
        }

        #[test]
        fn sort_components_with_equal_length() {
            let component_a = VersionComponent {
                value: 5,
                length: 2,
            };

            let component_b = VersionComponent {
                value: 7,
                length: 2,
            };

            assert_eq!(
                component_a.partial_cmp(&component_b),
                Some(std::cmp::Ordering::Less)
            )
        }

        #[test]
        fn sort_components_with_different_length() {
            let component_a = VersionComponent {
                value: 5,
                length: 2,
            };

            let component_b = VersionComponent {
                value: 7,
                length: 1,
            };

            assert_eq!(component_a.partial_cmp(&component_b), None)
        }
    }
}
