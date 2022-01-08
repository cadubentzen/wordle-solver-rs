use std::io::Write;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> std::io::Result<()> {
    let mut output = File::create("words5.txt")?;
    let input = File::open("words_alpha.txt")?;
    let input = BufReader::new(input);

    input
        .lines()
        .filter_map(|word| match word.as_ref() {
            Ok(word) => {
                if word.len() == 5 {
                    Some(word.to_string())
                } else {
                    None
                }
            }
            _ => None,
        })
        .for_each(|word| {
            writeln!(&mut output, "{}", word).unwrap();
        });

    Ok(())
}
