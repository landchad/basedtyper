use std::{fs, path::Path};
use std::io;

pub fn parse<T: AsRef<Path>>(path: T) -> Result<Vec<(String, String)>, io::Error> {
    let file = fs::read_to_string(path);

    if let Err(err) = file {
        return Err(err);
    } else {
        let file = file.unwrap();

        let chunks: Vec<&str> = file.split("\n\n").collect();

        let mut words: Vec<(String, String)> = vec![];

        chunks.iter().for_each(|chunk| {
            let word: Vec<&str> = chunk.split("\n").collect();
            words.push((String::from(word[0]), String::from(word[1])));
        });

        return Ok(words);
    }
}
