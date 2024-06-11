use std::{path::PathBuf, str::FromStr};

use time::{
    macros::{format_description, offset},
    OffsetDateTime, PrimitiveDateTime,
};

#[derive(Debug, PartialEq, Eq)]
enum Mood {
    None,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl FromStr for Mood {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-1" => Ok(Mood::None),
            "1" => Ok(Mood::One),
            "2" => Ok(Mood::Two),
            "3" => Ok(Mood::Three),
            "4" => Ok(Mood::Four),
            "5" => Ok(Mood::Five),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MoodLog {
    datetime: time::OffsetDateTime,
    mood: Mood,
    day_one_markdown_content: String,
    attachments: Vec<PathBuf>,
}

/// Parses the datetime from a line like "Date (human): 1970-01-01 00:01".
fn parse_datetime(datetime_line: &str) -> OffsetDateTime {
    let datetime = datetime_line.strip_prefix("Date (human): ").unwrap();
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
    PrimitiveDateTime::parse(datetime, &format)
        .unwrap()
        // TODO: Hardcoded
        .assume_offset(offset!(+1))
}

/// Parses a path from something like "![Untitled](ML%201970-01-01%2000%2000%2001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png)".
fn parse_attachment(line: &str) -> PathBuf {
    let start = line.find('(').unwrap() + 1;
    let end = line.find(')').unwrap();
    let path = &line[start..end];
    PathBuf::from(urlencoding::decode(path).unwrap().into_owned())
}

pub fn parse_file(content: &str) -> MoodLog {
    let mut lines = content.lines();
    lines.next(); // Skip the title
    lines.next(); // Skip the empty line
    let datetime = {
        let line = lines.next().unwrap();
        parse_datetime(line)
    };
    let mood: Mood = {
        let line = lines.next().unwrap();
        line.strip_prefix("Mood: ").unwrap().parse().unwrap()
    };
    lines.next(); // Skip the Date line
    lines.next(); // Skip the empty line

    let mut day_one_markdown_content = String::new();
    let mut attachments = vec![];
    for line in lines {
        if line.trim().starts_with("![") {
            attachments.push(parse_attachment(line.trim()));
            day_one_markdown_content.push_str("[{attachment}]");
        } else {
            day_one_markdown_content.push_str(line);
        }
        day_one_markdown_content.push('\n');
    }
    assert_eq!(day_one_markdown_content.pop().unwrap(), '\n'); // Remove the trailing newline

    MoodLog {
        datetime,
        mood,
        day_one_markdown_content,
        attachments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let content = "# ML 1970-01-01 00:01\n\nDate (human): 1970-01-01 01:01\nMood: -1\nDate: 1970/01/01 01:01 (GMT+1)\n\nHello, world!";
        assert_eq!(
            parse_file(content),
            MoodLog {
                datetime: time::OffsetDateTime::from_unix_timestamp(60).unwrap(),
                mood: Mood::None,
                day_one_markdown_content: "Hello, world!".to_owned(),
                attachments: vec![],
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_mood() {
        let content = "# ML 1970-01-01 00:01\n\nDate (human): 1970-01-01 01:01\nMood: -2\nDate: 1970/01/01 01:01 (GMT+1)\n\nHello, world!";
        parse_file(content);
    }

    #[test]
    fn test_file() {
        let content = "# ML 1970-01-01 00:01\n\nDate (human): 1970-01-01 01:01\nMood: 2\nDate: 1970/01/01 01:01 (GMT+1)\n\nHello, world! This is my image:\n![Untitled](ML%201970-01-01%2000%2000%2001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png)";
        let day_one_markdown_content = "Hello, world! This is my image:\n[{attachment}]".to_owned();
        assert_eq!(
            parse_file(content),
            MoodLog {
                datetime: time::OffsetDateTime::from_unix_timestamp(60).unwrap(),
                mood: Mood::Two,
                day_one_markdown_content,
                attachments: vec![PathBuf::from(
                    "ML 1970-01-01 00 00 01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png"
                )],
            }
        );
    }

    #[test]
    fn test_file_indented() {
        let content = "# ML 1970-01-01 00:01\n\nDate (human): 1970-01-01 01:01\nMood: 2\nDate: 1970/01/01 01:01 (GMT+1)\n\n- Hello, world! This is my image:\n\n    ![Untitled](ML%201970-01-01%2000%2000%2001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png)\n\n    - Hello again!";
        let day_one_markdown_content =
            "- Hello, world! This is my image:\n\n[{attachment}]\n\n    - Hello again!".to_owned();
        assert_eq!(
            parse_file(content),
            MoodLog {
                datetime: time::OffsetDateTime::from_unix_timestamp(60).unwrap(),
                mood: Mood::Two,
                day_one_markdown_content,
                attachments: vec![PathBuf::from(
                    "ML 1970-01-01 00 00 01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png"
                )],
            }
        );
    }

    #[test]
    fn test_two_files() {
        let content = "# ML 1970-01-01 00:02\n\nDate (human): 1970-01-01 01:02\nMood: 5\nDate: 1970/01/01 01:02 (GMT+1)\n\nHello, world! This is my image:\n![Untitled](ML%201970-01-01%2000%2000%2002aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png)\nAnd this is my other one:\n![Titled](ML%201970-01-01%2000%2000%2002aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Titled.png)";
        let day_one_markdown_content = "Hello, world! This is my image:\n[{attachment}]\nAnd this is my other one:\n[{attachment}]".to_owned();
        assert_eq!(
            parse_file(content),
            MoodLog {
                datetime: time::OffsetDateTime::from_unix_timestamp(120).unwrap(),
                mood: Mood::Five,
                day_one_markdown_content,
                attachments: vec![
                    PathBuf::from(
                        "ML 1970-01-01 00 00 02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png"
                    ),
                    PathBuf::from(
                        "ML 1970-01-01 00 00 02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Titled.png"
                    )
                ],
            }
        );
    }

    #[test]
    fn test_datetime_parse() {
        assert_eq!(
            parse_datetime("Date (human): 1970-01-01 01:01"),
            time::OffsetDateTime::from_unix_timestamp(60).unwrap()
        )
    }

    #[test]
    fn test_attachment_parse() {
        assert_eq!(
            parse_attachment("![Untitled](ML%201970-01-01%2000%2000%2001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png)"),
            PathBuf::from("ML 1970-01-01 00 00 01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Untitled.png")
        );
    }
}
