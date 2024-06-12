mod day_one;
mod notion;

fn main() {
    // Because we use `fs::canonicalize`, start in the folder with `*.md`
    for file in glob::glob("ML *.md").unwrap() {
        let file = file.unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        let mood_log = notion::parse_file(&content);
        let mood_tag = match mood_log.mood {
            notion::Mood::None => "mood/-1",
            notion::Mood::One => "mood/1",
            notion::Mood::Two => "mood/2",
            notion::Mood::Three => "mood/3",
            notion::Mood::Four => "mood/4",
            notion::Mood::Five => "mood/5",
        }
        .to_owned();
        day_one::make_entry(
            mood_log.day_one_markdown_content,
            mood_log.attachments,
            vec![mood_tag, "from-notion".to_owned()],
            // Hardcoded
            Some("Journal".to_owned()),
            Some(mood_log.datetime),
            false,
        );
        println!("Processed {:?}", file);
    }
}
