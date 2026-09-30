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
        // the separator is whatever this platform uses. A single segment
        // literally named "2024/03/14" would fail here.
        assert_eq!(rendered, PathBuf::from("2024").join("03").join("14"));

        // And it read the modified date, not the created one (2023/11/02).
    }

    #[test]
    fn test_render_folder_rejects_an_unknown_token() {
        // arrange
        let destination = destination_with("{colour}");
        let file = sample_file();

        // act
        let rendered = destination.render_folder(&file);

        // assert — an unknown token is an error the dry run can show, never
        // a panic and never the braces left in the path.
        assert!(rendered.is_err());
    }

    // Still to write, once the two above are green:
    //
    //   - literal text survives:      "photos/{year}"  ->  photos/2024
    //   - the remaining tokens:       "{name}.{ext}"   ->  IMG_4471.heic
    //   - an unclosed brace is an error:  "{year"
    //   - an empty token is an error:     "{}"
    //   - tokens that arrive later are errors today: {type}, {size-band},
    //     {parent}, {counter}, {match} — this is what tells you the parser
    //     still rejects them properly once p1-type adds the first two.
    //   - a missing value: {ext} on a file whose extension is None.
    //     Decide first: error out, or substitute a folder name? Erroring
    //     fails a whole run over one odd file; substituting puts it in
    //     something like "no-extension". Write the test once you have picked.
    //   - an empty pattern: does it mean the destination root itself?
}
