use super::app::App;
use io::Write;
use rand::Rng;
use serde_json::Value;
use std::{fs::{self, OpenOptions}, io, path::Path};

pub struct Word {
    word: String,
    definition: String,
}

impl Word {
    pub fn new<T: ToString>(word: T, definition: T) -> Self {
        let word = word.to_string();
        let definition = definition.to_string();

        Self {
            word,
            definition,
        }
    }

    pub fn get_definition(&self) -> &String {
        &self.definition
    }

    pub fn get_word(&self) -> &String {
        &self.word
    }
}

pub fn parse_words(mode: &str, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.words = Vec::new();
    match mode {
        "quote" => {
            let other_res = ureq::get("https://www.reddit.com/r/copypasta/top/.json?sort=top&t=week&showmedia=false&mediaonly=false&is_self=true&limit=100")
                .call()?.into_string()?;

            let json: Value = serde_json::from_str(&other_res[..]).unwrap();

            let quote = json["data"]["children"][rand::thread_rng().gen_range(0..100) as usize]["data"]["selftext"].as_str().unwrap();
            
            for word in quote.split(" ") {
                app.words.push(Word::new(word, ""));
            }

            if app.config.cache_quotes {
                let home_dir = std::env::var("HOME").unwrap();

                let words_vec = app.words
                    .iter()
                    .map(|elem| elem.get_word().into())
                    .collect::<Vec<String>>();

                let word_string = words_vec.join(" ");


                /*if !Path::new(&format!("{}/.cache/basedtyper.cached_quotes", home_dir)).is_file() {
                    std::fs::File::create(format!("{}/.cache/basedtyper.cached_quotes", home_dir)).unwrap();
                }*/

                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(format!("{}/.cache/basedtyper.cached_quotes", home_dir))
                    .unwrap();

                file.write((word_string + "\n\n").as_bytes()).unwrap();
            }

            Ok(())
        }

        _ => {
            let parsed_words = parse_wordlist(app.locate_wordlist(&app.wordlist.1), &10, &app);

            if parsed_words.is_ok() {
                app.words = parsed_words.unwrap();
                
                return Ok(());
            }

            Err(Box::new(parsed_words.err().unwrap()))
        }
    }
}

pub fn parse_wordlist<T: AsRef<Path>>(path: T, count: &u32, app: &App) -> Result<Vec<Word>, io::Error> {
    let file = fs::read_to_string(path);

    if let Err(err) = file {
        Err(err)
    } else {
        let file = file.unwrap();

        let chunks: Vec<&str> = file.split("\n\n").collect();

        let mut words: Vec<Word> = vec![];

        for _ in if *count > chunks.len() as u32 {
            0..chunks.len() as u32
        } else { 
            0..*count
        } {
            let rand = rand::thread_rng().gen_range(0..chunks.len());
            let word: Vec<&str> = chunks[rand].split('\n').collect();

            if !word[0].starts_with("#") {
                if !app.config.definitions {
                    words.push(Word::new(word[0], ""));
                } else {
                    words.push(Word::new(word[0], word[1]));
                }
            }
        }

        Ok(words)
    }
}