#![forbid(unsafe_code)]

use goreecloud_download_core::RemoteValidators;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IfRangeValidator {
    StrongEtag(String),
    LastModified(String),
}

impl IfRangeValidator {
    pub fn as_header_value(&self) -> &str {
        match self {
            Self::StrongEtag(value) | Self::LastModified(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpRequestPlan {
    Full,
    RestartFull {
        discarded_partial_bytes: u64,
    },
    Resume {
        start_at: u64,
        if_range: IfRangeValidator,
    },
}

impl HttpRequestPlan {
    pub fn range_header_value(&self) -> Option<String> {
        match self {
            Self::Resume { start_at, .. } => Some(format!("bytes={start_at}-")),
            Self::Full | Self::RestartFull { .. } => None,
        }
    }

    pub fn if_range_header_value(&self) -> Option<&str> {
        match self {
            Self::Resume { if_range, .. } => Some(if_range.as_header_value()),
            Self::Full | Self::RestartFull { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteContentRange {
    pub start: u64,
    pub end: u64,
    pub complete_length: Option<u64>,
}

impl ByteContentRange {
    pub fn is_well_formed(&self) -> bool {
        if self.end < self.start {
            return false;
        }
        self.complete_length.is_none_or(|length| self.end < length)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpResponseMetadata {
    pub status_code: u16,
    pub content_range: Option<ByteContentRange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpResponseDecision {
    AcceptFullBody,
    AppendResumeBody,
    RetryFromBeginning,
    Reject,
}

pub fn plan_request(downloaded_bytes: u64, validators: &RemoteValidators) -> HttpRequestPlan {
    if downloaded_bytes == 0 {
        return HttpRequestPlan::Full;
    }

    if let Some(etag) = validators
        .etag
        .as_deref()
        .filter(|value| is_strong_etag(value))
    {
        return HttpRequestPlan::Resume {
            start_at: downloaded_bytes,
            if_range: IfRangeValidator::StrongEtag(etag.to_owned()),
        };
    }

    if let Some(last_modified) = validators
        .last_modified
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return HttpRequestPlan::Resume {
            start_at: downloaded_bytes,
            if_range: IfRangeValidator::LastModified(last_modified.to_owned()),
        };
    }

    HttpRequestPlan::RestartFull {
        discarded_partial_bytes: downloaded_bytes,
    }
}

pub fn evaluate_response(
    plan: &HttpRequestPlan,
    response: HttpResponseMetadata,
) -> HttpResponseDecision {
    match plan {
        HttpRequestPlan::Full | HttpRequestPlan::RestartFull { .. } => {
            if response.status_code == 200 {
                HttpResponseDecision::AcceptFullBody
            } else {
                HttpResponseDecision::Reject
            }
        }
        HttpRequestPlan::Resume { start_at, .. } => match response.status_code {
            200 => HttpResponseDecision::AcceptFullBody,
            206 => match response.content_range {
                Some(range) if range.is_well_formed() && range.start == *start_at => {
                    HttpResponseDecision::AppendResumeBody
                }
                _ => HttpResponseDecision::Reject,
            },
            412 | 416 => HttpResponseDecision::RetryFromBeginning,
            _ => HttpResponseDecision::Reject,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentRangeParseError {
    InvalidFormat,
    UnsupportedUnit,
    InvalidNumber,
    InvalidBounds,
}

pub fn parse_content_range(value: &str) -> Result<ByteContentRange, ContentRangeParseError> {
    let mut fields = value.split_ascii_whitespace();
    let unit = fields.next().ok_or(ContentRangeParseError::InvalidFormat)?;
    let range_and_length = fields.next().ok_or(ContentRangeParseError::InvalidFormat)?;
    if fields.next().is_some() {
        return Err(ContentRangeParseError::InvalidFormat);
    }
    if !unit.eq_ignore_ascii_case("bytes") {
        return Err(ContentRangeParseError::UnsupportedUnit);
    }

    let (range, complete_length) = range_and_length
        .split_once('/')
        .ok_or(ContentRangeParseError::InvalidFormat)?;
    let (start, end) = range
        .split_once('-')
        .ok_or(ContentRangeParseError::InvalidFormat)?;

    let start = start
        .parse::<u64>()
        .map_err(|_| ContentRangeParseError::InvalidNumber)?;
    let end = end
        .parse::<u64>()
        .map_err(|_| ContentRangeParseError::InvalidNumber)?;
    let complete_length = if complete_length == "*" {
        None
    } else {
        Some(
            complete_length
                .parse::<u64>()
                .map_err(|_| ContentRangeParseError::InvalidNumber)?,
        )
    };

    let parsed = ByteContentRange {
        start,
        end,
        complete_length,
    };
    if !parsed.is_well_formed() {
        return Err(ContentRangeParseError::InvalidBounds);
    }

    Ok(parsed)
}

fn is_strong_etag(value: &str) -> bool {
    let value = value.trim();
    value.len() >= 2
        && value.starts_with('"')
        && value.ends_with('"')
        && !value.to_ascii_lowercase().starts_with("w/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validators(etag: Option<&str>, last_modified: Option<&str>) -> RemoteValidators {
        RemoteValidators {
            etag: etag.map(str::to_owned),
            last_modified: last_modified.map(str::to_owned),
        }
    }

    #[test]
    fn content_range_parser_accepts_known_and_unknown_complete_lengths() {
        assert_eq!(
            parse_content_range("bytes 1024-2047/4096"),
            Ok(ByteContentRange {
                start: 1024,
                end: 2047,
                complete_length: Some(4096),
            })
        );
        assert_eq!(
            parse_content_range("BYTES 5-9/*"),
            Ok(ByteContentRange {
                start: 5,
                end: 9,
                complete_length: None,
            })
        );
    }

    #[test]
    fn content_range_parser_rejects_malformed_or_impossible_ranges() {
        assert_eq!(
            parse_content_range("items 0-1/2"),
            Err(ContentRangeParseError::UnsupportedUnit)
        );
        assert_eq!(
            parse_content_range("bytes 10-9/20"),
            Err(ContentRangeParseError::InvalidBounds)
        );
        assert_eq!(
            parse_content_range("bytes 0-20/20"),
            Err(ContentRangeParseError::InvalidBounds)
        );
        assert_eq!(
            parse_content_range("bytes nope"),
            Err(ContentRangeParseError::InvalidFormat)
        );
    }

    #[test]
    fn zero_progress_uses_full_request() {
        assert_eq!(
            plan_request(0, &validators(Some("\"v1\""), None)),
            HttpRequestPlan::Full
        );
    }

    #[test]
    fn partial_transfer_uses_strong_etag_for_if_range() {
        let plan = plan_request(4096, &validators(Some("\"v1\""), None));
        assert_eq!(
            plan,
            HttpRequestPlan::Resume {
                start_at: 4096,
                if_range: IfRangeValidator::StrongEtag("\"v1\"".into()),
            }
        );
        assert_eq!(plan.range_header_value().as_deref(), Some("bytes=4096-"));
        assert_eq!(plan.if_range_header_value(), Some("\"v1\""));
    }

    #[test]
    fn weak_etag_falls_back_to_last_modified() {
        assert_eq!(
            plan_request(
                512,
                &validators(Some("W/\"weak\""), Some("Tue, 15 Sep 2026 12:00:00 GMT"))
            ),
            HttpRequestPlan::Resume {
                start_at: 512,
                if_range: IfRangeValidator::LastModified("Tue, 15 Sep 2026 12:00:00 GMT".into()),
            }
        );
    }

    #[test]
    fn partial_transfer_without_safe_validator_restarts_full() {
        assert_eq!(
            plan_request(8192, &validators(Some("W/\"weak\""), None)),
            HttpRequestPlan::RestartFull {
                discarded_partial_bytes: 8192,
            }
        );
    }

    #[test]
    fn matching_partial_content_range_can_append() {
        let plan = plan_request(1024, &validators(Some("\"v1\""), None));
        let response = HttpResponseMetadata {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 1024,
                end: 2047,
                complete_length: Some(4096),
            }),
        };
        assert_eq!(
            evaluate_response(&plan, response),
            HttpResponseDecision::AppendResumeBody
        );
    }

    #[test]
    fn mismatched_partial_content_range_is_rejected() {
        let plan = plan_request(1024, &validators(Some("\"v1\""), None));
        let response = HttpResponseMetadata {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 0,
                end: 1023,
                complete_length: Some(4096),
            }),
        };
        assert_eq!(
            evaluate_response(&plan, response),
            HttpResponseDecision::Reject
        );
    }

    #[test]
    fn malformed_partial_content_range_is_rejected() {
        let plan = plan_request(1024, &validators(Some("\"v1\""), None));
        let response = HttpResponseMetadata {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 1024,
                end: 4096,
                complete_length: Some(4096),
            }),
        };
        assert_eq!(
            evaluate_response(&plan, response),
            HttpResponseDecision::Reject
        );
    }

    #[test]
    fn full_body_response_to_resume_must_replace_partial_data() {
        let plan = plan_request(1024, &validators(Some("\"v1\""), None));
        assert_eq!(
            evaluate_response(
                &plan,
                HttpResponseMetadata {
                    status_code: 200,
                    content_range: None,
                }
            ),
            HttpResponseDecision::AcceptFullBody
        );
    }

    #[test]
    fn precondition_or_range_failure_retries_from_beginning() {
        let plan = plan_request(1024, &validators(Some("\"v1\""), None));
        for status_code in [412, 416] {
            assert_eq!(
                evaluate_response(
                    &plan,
                    HttpResponseMetadata {
                        status_code,
                        content_range: None,
                    }
                ),
                HttpResponseDecision::RetryFromBeginning
            );
        }
    }

    #[test]
    fn unexpected_partial_response_to_full_request_is_rejected() {
        assert_eq!(
            evaluate_response(
                &HttpRequestPlan::Full,
                HttpResponseMetadata {
                    status_code: 206,
                    content_range: Some(ByteContentRange {
                        start: 0,
                        end: 9,
                        complete_length: Some(10),
                    }),
                }
            ),
            HttpResponseDecision::Reject
        );
    }
}
