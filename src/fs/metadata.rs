use base64::Engine as _;
use rocket::serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt::Display};

/// A struct representing the metadata associated with an uploaded file.
///
/// Metadata is a wrapper around a `HashMap` that holds metadata for a tus upload.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Metadata(HashMap<String, String>);

/// An error type representing errors that can occur while dealing with metadata.
#[derive(Debug, PartialEq)]
pub enum MetadataError {
    /// An error indicating an invalid key was used while getting raw metadata.
    InvalidKey,
    /// An error indicating a base64 decode error occurred while getting raw metadata.
    DecodeError(String),
    /// An error indicating the metadata string was invalid.
    InvalidMetadataFormat,
}

impl Error for MetadataError {}

impl Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Metadata {
    /// Creates a new empty metadata.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the raw binary value of the metadata associated with the given key.
    ///
    /// This function returns an error if the given key is not present in the metadata, or if the value of the
    /// entry could not be decoded from Base64. Otherwise, it returns the decoded binary value as a vector of bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use meteoritus::{Metadata, MetadataError};
    /// use std::str::from_utf8;
    ///
    /// let metadata = Metadata::try_from("filetype dmlkZW8vbXA0, filename bXlfdmlkZW8ubXA0").unwrap();
    ///
    /// assert_eq!(metadata.get_raw("filetype"), Ok(b"video/mp4".to_vec()));
    /// assert_eq!(from_utf8(&metadata.get_raw("filename").unwrap()), Ok("my_video.mp4"));
    /// assert_eq!(metadata.get_raw("foo bar bars"), Err(MetadataError::InvalidKey));
    ///
    /// ```
    pub fn get_raw(&self, key: &str) -> Result<Vec<u8>, MetadataError> {
        let value = match self.0.get(key) {
            Some(v) => v,
            None => return Err(MetadataError::InvalidKey),
        };

        match base64::engine::general_purpose::STANDARD.decode(value) {
            Ok(decoded) => Ok(decoded),
            Err(e) => Err(MetadataError::DecodeError(e.to_string())),
        }
    }

    /// Returns the number of elements in the metadata.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl TryFrom<&str> for Metadata {
    type Error = MetadataError;

    /// Attempts to parse a metadata string into a [`Metadata`] instance.
    ///
    /// The given string should follow the tus [`Upload-Metadata`](https://tus.io/protocols/resumable-upload.html#upload-metadata) definition .
    ///
    /// # Examples
    ///
    /// ```
    /// use meteoritus::{Metadata, MetadataError};
    ///
    ///let metadata = Metadata::try_from("relativePath bnVsbA==, filetype dmlkZW8vbXA0,is_confidential").unwrap();
    ///assert_eq!(metadata.len(), 3);
    ///
    ///let metadata = Metadata::try_from("");
    ///assert!(metadata.is_err());
    ///assert_eq!(metadata.err(), Some(MetadataError::InvalidMetadataFormat));
    ///
    ///let metadata = Metadata::try_from("foobar, fas bars foo bar, ");
    ///assert!(metadata.is_err());
    ///assert_eq!(metadata.err(), Some(MetadataError::InvalidMetadataFormat));
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(MetadataError::InvalidMetadataFormat);
        }

        let mut metadata = Metadata::new();

        for pair in value.split(',') {
            let pair = pair.trim();

            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.split(' ').map(|v| v.trim()).collect();

            if parts.is_empty() || parts.len() > 2 {
                return Err(MetadataError::InvalidMetadataFormat);
            }

            if parts[0].is_empty() {
                return Err(MetadataError::InvalidKey);
            }

            if let (Some(key), value) = (parts.get(0), parts.get(1)) {
                let value = match value {
                    Some(v) => v.to_string(),
                    None => String::default(),
                };

                metadata.0.insert(key.to_string(), value);
            }
        }

        Ok(metadata)
    }
}
