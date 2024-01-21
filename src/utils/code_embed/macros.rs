macro_rules! assert_correct_domain {
    ($url:expr, $expected:expr) => {{
        if let Some(domain) = $url.domain() {
            if domain != $expected {
                return Err(ParserError::WrongParser(
                    $expected.to_string(),
                    domain.to_string(),
                ));
            }
        } else {
            return Err(ParserError::Error("No domain found".to_string()));
        }
    }};
}

macro_rules! parse_segment {
    ($segments:expr, $segment:tt) => {
        $segments.next().ok_or(ParserError::ConversionError(format!(
            "Failed to parse {}",
            $segment.to_string()
        )))
    };
}

pub(crate) use {assert_correct_domain, parse_segment};
