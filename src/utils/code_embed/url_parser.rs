use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use poise::async_trait;
use reqwest::Url;

use super::macros::parse_segment;
use crate::utils::code_embed::macros::assert_correct_domain;

/// A struct that represents a GitHub code URL.
///
/// **Note**: The domain of the url has to be github.com.
pub struct GitHubCodeUrl {
    pub url: Url,
}

/// A struct that holds details on code.
#[derive(Default, std::fmt::Debug)]
pub struct CodeUrl {
    pub raw_code_url: String,
    pub original_code_url: String,
    pub repo: String,
    pub user: String,
    pub branch_or_sha: String,
    pub relevant_lines: Option<(usize, usize)>,
    pub language: Option<String>,
}

/// A struct that holds details on code and a code preview.
#[derive(Default, std::fmt::Debug)]
pub struct CodePreview {
    pub code: CodeUrl,
    pub preview: Option<String>,
}

#[async_trait]
pub trait CodeUrlParser {
    fn kind(&self) -> &'static str;
    async fn parse(&self) -> Result<CodePreview, ParserError>;
    fn parse_code_url(&self) -> Result<CodeUrl, ParserError>;
}

#[async_trait]
impl CodeUrlParser for GitHubCodeUrl {
    fn kind(&self) -> &'static str {
        "github.com"
    }

    fn parse_code_url(&self) -> Result<CodeUrl, ParserError> {
        let mut segments = self
            .url
            .path_segments()
            .ok_or(ParserError::ConversionError(
                "Failed to convert path segments".to_string(),
            ))?;

        // parse the segments

        let user = parse_segment!(segments, "user")?;
        let repo = parse_segment!(segments, "repo")?;
        let _blob_segment = parse_segment!(segments, "blob"); // GitHub specific segment
        let branch_or_sha = parse_segment!(segments, "branch or sha")?;

        let mut path = String::new();
        while let Ok(segment) = parse_segment!(segments, "path") {
            if segment == "" {
                continue;
            }
            path.push('/');
            path.push_str(segment);
        }

        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}{}",
            user, repo, branch_or_sha, path
        );

        let mut code_url = CodeUrl {
            raw_code_url: raw_url,
            original_code_url: self.url.to_string(),
            repo: repo.to_string(),
            user: user.to_string(),
            branch_or_sha: branch_or_sha.to_string(),
            ..Default::default()
        };

        if let Some(fragment) = self.url.fragment() {
            let mut numbers = fragment
                .split('-')
                .map(|s| s.trim_matches('L'))
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| ParserError::InvalidFragment(fragment.to_string()))?;

            if numbers.len() > 2 {
                return Err(ParserError::InvalidFragment(fragment.to_string()));
            }

            let start = numbers.remove(0);
            let end = numbers.pop().unwrap_or_else(|| start);
            code_url.relevant_lines = Some((start, end));
        }

        let mut segments = self.url.path_segments().unwrap();
        while let Some(segment) = segments.next_back() {
            if !segment.is_empty() {
                let extension = Path::new(segment)
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(str::to_string);
                code_url.language = extension;

                break;
            }
        }
        Ok(code_url)
    }

    async fn parse(&self) -> Result<CodePreview, ParserError> {
        assert_correct_domain!(self.url, self.kind());

        let code_url = self.parse_code_url()?;

        // TODO: If the code is huge, downloading could take long. If code_url.relevant_lines is Some, only download up to the relevant lines.
        let code = reqwest::get(&code_url.raw_code_url)
            .await
            .map_err(|_| ParserError::FailedToGetCode("Can't make a request".to_string()))?
            .text()
            .await
            .map_err(|_| ParserError::FailedToGetCode("Can't parse body".to_string()))?;

        let preview = if let Some((start, end)) = code_url.relevant_lines.clone() {
            let lines = code.lines().collect::<Vec<_>>();
            let start = start - 1;
            let end = end - 1;

            if start > end || start >= lines.len() || end >= lines.len() {
                return Err(ParserError::InvalidFragment(format!("{}-{}", start, end)));
            }

            let mut code_block = String::new();

            code_block.push_str("```");

            if let Some(language) = code_url.language.clone() {
                code_block.push_str(&language);
                code_block.push('\n');
            }

            code_block.push_str(&lines[start..=end].join("\n"));
            code_block.push_str("```");

            Some(code_block)
        } else {
            None
        };

        let code_preview = CodePreview {
            code: code_url,
            preview,
        };

        Ok(code_preview)
    }
}

#[derive(Debug, Clone)]
pub enum ParserError {
    Error(String),
    WrongParserError(String, String),
    ConversionError(String),
    InvalidFragment(String),
    FailedToGetCode(String),
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::Error(e) => {
                write!(f, "Error: {}", e)
            },
            ParserError::WrongParserError(expected, got) => {
                write!(f, "Expected parser {}, got {}", expected, got)
            },
            ParserError::ConversionError(conversion_error) => {
                write!(f, "Conversion error: {}", conversion_error)
            },
            ParserError::InvalidFragment(fragment) => {
                write!(f, "Invalid fragment: {}", fragment)
            },
            ParserError::FailedToGetCode(error) => {
                write!(f, "Failed to get code: {}", error)
            },
        }
    }
}

impl From<Box<dyn std::error::Error>> for ParserError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::Error(e.to_string())
    }
}
