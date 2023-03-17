use std::collections::HashMap;

#[derive(Default)]
pub struct Metadata(HashMap<String, String>);

impl Metadata {
    pub fn new() -> Self {
        Default::default()
    }
}

impl TryFrom<&str> for Metadata {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Metadata cannot be empty");
        }

        let mut metadata = Metadata::new();

        for pair in value.split(',') {
            let pair = pair.trim();

            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.split(' ').map(|v| v.trim()).collect();

            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err("Invalid metadata format");
            }

            if let [key, value] = parts[..] {
                metadata.0.insert(key.to_string(), value.to_string());
            }
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA_STR: &str =
        "relativePath bnVsbA==, filename bXlfdmlkZW8ubXA0, filetype dmlkZW8vbXA0";

    #[test]
    fn valid_from_str() {
        let metadata = Metadata::try_from(METADATA_STR).unwrap();

        assert_eq!(metadata.0.len(), 3);
        assert_eq!(
            metadata.0.get("relativePath"),
            Some(&String::from("bnVsbA=="))
        );
        assert_eq!(
            metadata.0.get("filetype"),
            Some(&String::from("dmlkZW8vbXA0"))
        );
        assert_eq!(
            metadata.0.get("filename"),
            Some(&String::from("bXlfdmlkZW8ubXA0"))
        );
    }

    #[test]
    fn empty_from_str_error() {
        let metadata = Metadata::try_from("");

        assert!(metadata.is_err());
        assert_eq!(metadata.err(), Some("Metadata cannot be empty"));
    }

    #[test]
    fn invalid_format_from_str_error() {
        let metadata = Metadata::try_from("foobar, fas bars foo bar, ");

        assert!(metadata.is_err());
        assert_eq!(metadata.err(), Some("Invalid metadata format"));
    }
}
