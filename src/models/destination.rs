#![allow(dead_code)]
use crate::models::error::pattern_error::PatternError;
use crate::models::file_item::FileItem;
use crate::models::token::Token;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rename {
    pub pattern: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Destination {
    pub folder: PathBuf,
    pub sub_folder_pattern: String,
    pub rename: Option<Rename>,
}

impl Destination {
    pub fn render_folder(&self, file_item: &FileItem) -> Result<PathBuf, PatternError> {
        let mut rendered = String::new();
        let mut rest = self.sub_folder_pattern.as_str();
        let mut path = PathBuf::new();

        while let Some(start) = rest.find('{') {
            rendered.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let close = after
                .find('}')
                .ok_or_else(|| PatternError::InvalidPattern(rest.to_string()))?;
            let tag = &after[..close];

            match Token::parse(tag) {
                Some(token) => {
                    rendered.push_str(&token.value(file_item));
                }
                None => return Err(PatternError::UnsupportedToken(tag.to_string())),
            }
            rest = &after[close + 1..];
        }
        rendered.push_str(rest);
        rendered
            .split('/')
            .filter(|s| !s.is_empty())
            .for_each(|folder| path.push(folder));
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::file_item::FileItem;
    use chrono::NaiveDate;

    fn sample_file() -> FileItem {
        FileItem {
            path: PathBuf::from("/source/IMG_4471.HEIC"),
            name: "IMG_4471".to_string(),
            extension: Some("heic".to_string()),
            size: 4_200_000,
            destination: None,
            created_at: NaiveDate::from_ymd_opt(2023, 11, 2),
            modified_at: NaiveDate::from_ymd_opt(2024, 3, 14),
        }
    }

    fn destination_with(pattern: &str) -> Destination {
        Destination {
            folder: PathBuf::from("/pictures"),
            sub_folder_pattern: pattern.to_string(),
            rename: None,
        }
    }

    #[test]
    fn test_render_folder_substitutes_tokens() {
        // arrange
        let destination = destination_with("{year}/{month}/{day}");
        let file = sample_file();

        // act
        let rendered = destination.render_folder(&file).unwrap();

        // assert — built with `join` rather than a "2024/03/14" literal, so
        assert_eq!(rendered, PathBuf::from("2024").join("03").join("14"));
    }

    #[test]
    fn test_render_folder_rejects_an_unknown_token() {
        // arrange
        let destination = destination_with("{colour}");
        let file = sample_file();

        // act
        let rendered = destination.render_folder(&file);

        // assert
        assert_eq!(
            rendered,
            Err(PatternError::UnsupportedToken("colour".into()))
        )
    }

    #[test]
    fn test_render_folder_rejects_an_unclosed_brace() {
        // arrange
        let destination = destination_with("{year");
        let file = sample_file();

        // act
        let rendered = destination.render_folder(&file);

        // assert
        assert_eq!(rendered, Err(PatternError::InvalidPattern("{year".into())))
    }

    #[test]
    fn test_render_folder_text_survives_with_tags() {
        let destination = destination_with("photos/{year}");
        let file = sample_file();

        let rendered = destination.render_folder(&file).unwrap();

        assert_eq!(rendered, PathBuf::from("photos").join("2024"));
    }

    #[test]
    fn test_render_folder_only_tags() {
        let destination = destination_with("{name}.{ext}");
        let file = sample_file();
        let rendered = destination.render_folder(&file).unwrap();
        assert_eq!(rendered, PathBuf::from("IMG_4471.heic"));
    }

    #[test]
    fn test_render_folder_rejects_empty_tags() {
        let destination = destination_with("{}.{}");
        let file = sample_file();
        let rendered = destination.render_folder(&file);
        assert_eq!(rendered, Err(PatternError::UnsupportedToken("".into())))
    }

    #[test]
    fn test_render_folder_rejects_unhandled_tags() {
        let destination = destination_with("{parent}");
        let file = sample_file();
        let rendered = destination.render_folder(&file);
        assert_eq!(
            rendered,
            Err(PatternError::UnsupportedToken("parent".into()))
        )
    }

    #[test]
    fn test_render_folder_dont_rejects_empty_values() {
        let destination = destination_with("{ext}");
        let file = FileItem {
            extension: None,
            ..sample_file()
        };
        let rendered = destination.render_folder(&file);
        assert_eq!(rendered.unwrap(), PathBuf::from("no-extension"));
    }

    #[test]
    fn test_render_folder_dont_rejects_empty_pattern() {
        let destination = destination_with("");
        let file = sample_file();
        let rendered = destination.render_folder(&file);
        assert_eq!(rendered.unwrap(), PathBuf::from(""));
    }

    #[test]
    fn test_value_all_types() {
        let file = sample_file();
        let destination = destination_with("{type}/{size-band}/{year}/{month}/{day}/{name}.{ext}");
        let rendered = destination.render_folder(&file).unwrap();
        assert_eq!(
            rendered,
            PathBuf::from("photos")
                .join("small")
                .join("2024")
                .join("03")
                .join("14")
                .join("IMG_4471.heic")
        );
    }
}
