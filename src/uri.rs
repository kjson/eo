use anyhow::Result;

/// Parses an S3 URI (e.g., s3://bucket/key) into a tuple of (bucket, key)
pub fn parse_uri(uri: &Option<String>) -> Result<Option<(String, String)>> {
    if let Some(uri_str) = uri {
        if let Some(stripped) = uri_str.strip_prefix("s3://") {
            if let Some((bucket, key)) = stripped.split_once('/') {
                return Ok(Some((bucket.to_string(), key.to_string())));
            }
        }
        return Err(anyhow::anyhow!(
            "Invalid S3 URI format. Expected s3://bucket/key"
        ));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri_valid() {
        let uri = Some(String::from("s3://mybucket/mykey"));
        let result = parse_uri(&uri).unwrap();
        assert_eq!(
            result,
            Some((String::from("mybucket"), String::from("mykey")))
        );
    }

    #[test]
    fn test_parse_uri_invalid_format() {
        let uri = Some(String::from("invalid://mybucket/mykey"));
        let result = parse_uri(&uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_uri_none() {
        let result = parse_uri(&None);
        assert_eq!(result.unwrap(), None);
    }
}
