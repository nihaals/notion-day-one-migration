use std::{fs, path::PathBuf, process::Command};

use time::macros::format_description;

fn format_datetime(datetime: time::OffsetDateTime) -> String {
    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    datetime
        .to_offset(time::UtcOffset::UTC)
        .format(format)
        .unwrap()
}

fn make_args(
    content: String,
    attachments: Vec<PathBuf>,
    tags: Vec<String>,
    journal: Option<String>,
    datetime: Option<time::OffsetDateTime>,
    starred: bool,
) -> Vec<String> {
    let mut args: Vec<String> = vec![];
    if !attachments.is_empty() {
        args.push("--attachments".to_owned());
        args.extend(
            attachments
                .iter()
                .map(|p| fs::canonicalize(p).unwrap().to_str().unwrap().to_owned()),
        );
    }
    if !tags.is_empty() {
        args.push("--tags".to_owned());
        args.extend(tags);
    }
    if let Some(journal) = journal {
        args.push("--journal".to_owned());
        args.push(journal);
    }
    if let Some(datetime) = datetime {
        args.push("--isoDate".to_owned());
        args.push(format_datetime(datetime));
    }
    if starred {
        args.push("--starred".to_owned());
    }
    if !args.is_empty() {
        args.push("--".to_owned());
    }
    args.push("new".to_owned());
    args.push(content);
    args
}

pub fn make_entry(
    content: String,
    attachments: Vec<PathBuf>,
    tags: Vec<String>,
    journal: Option<String>,
    datetime: Option<time::OffsetDateTime>,
    starred: bool,
) {
    let args = make_args(content, attachments, tags, journal, datetime, starred);
    let status = Command::new("dayone2")
        .args(args)
        .status()
        .expect("Failed to execute command");
    assert!(status.success());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_format() {
        assert_eq!(
            format_datetime(time::OffsetDateTime::from_unix_timestamp(1).unwrap()),
            "1970-01-01T00:00:01Z"
        )
    }

    #[test]
    fn test_all() {
        assert_eq!(
            make_args(
                "Entry content".to_owned(),
                vec![PathBuf::from("/bin/bash"), PathBuf::from("/bin/sh")],
                vec!["tag1".to_owned(), "tag2".to_owned()],
                Some("Journal".to_owned()),
                Some(time::OffsetDateTime::from_unix_timestamp(1).unwrap()),
                true
            ),
            vec![
                "--attachments",
                "/bin/bash",
                "/bin/sh",
                "--tags",
                "tag1",
                "tag2",
                "--journal",
                "Journal",
                "--isoDate",
                "1970-01-01T00:00:01Z",
                "--starred",
                "--",
                "new",
                "Entry content"
            ]
        )
    }
}
