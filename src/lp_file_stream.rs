extern crate deunicode;

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Lines},
};

use deunicode::deunicode;

pub struct LpFileStream {
    lines: Lines<BufReader<File>>,
}

#[derive(Debug, Clone)]
pub struct LpFileEntry {
    pub lang: String,
    pub country: String,
    pub tokens: Vec<LpEntryToken>,
}

#[derive(Debug, Clone)]
pub struct LpEntryToken {
    pub word: String,
    pub transliterated: String,
    pub label: String,
}

impl LpFileStream {
    pub fn new(filename: String) -> Result<LpFileStream, Box<dyn Error>> {
        let reader = BufReader::new(File::open(filename)?);

        Ok(LpFileStream {
            lines: reader.lines(),
        })
    }
}

impl Iterator for LpFileStream {
    type Item = LpFileEntry;

    fn next(&mut self) -> Option<LpFileEntry> {
        if let Some(Ok(next_line)) = self.lines.next() {
            let line_cells: Vec<&str> = next_line.split('\t').collect();
            if line_cells.len() != 3 {
                dbg!(
                    "Expected 3 cells in libpostal data but found {}",
                    line_cells.len()
                );
                return None;
            }
            let token_strings: Vec<&str> = line_cells[2].split(' ').collect();
            let tokens: Vec<LpEntryToken> = token_strings
                .iter()
                .filter_map(|token| {
                    if let Some((word, label)) = token.rsplit_once('/') {
                        let transliterated = deunicode(&word).to_ascii_lowercase();
                        let transliterated_tokens: Vec<&str> =
                            transliterated.split_ascii_whitespace().collect();
                        Some(LpEntryToken {
                            word: word.to_lowercase(),
                            label: label.to_string(),
                            transliterated: transliterated_tokens.join(""),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Some(LpFileEntry {
                lang: line_cells[0].to_string(),
                country: line_cells[1].to_string(),
                tokens,
            })
        } else {
            None
        }
    }
}
