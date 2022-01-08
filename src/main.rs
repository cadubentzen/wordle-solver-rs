use rand::prelude::*;
use std::collections::hash_set::Iter as HashSetIter;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::io::{self, BufRead};
use std::{fs::File, io::BufReader};
use structopt::StructOpt;

struct Words {
    words: Vec<String>,
    database: HashMap<char, HashMap<usize, HashSet<String>>>,
}

impl Words {
    fn new() -> io::Result<Self> {
        let words = BufReader::new(File::open("words5.txt")?)
            .lines()
            .collect::<io::Result<Vec<_>>>()?;

        let mut database = HashMap::<char, HashMap<usize, HashSet<String>>>::new();

        for word in &words {
            for (i, letter) in word.chars().enumerate() {
                database
                    .entry(letter)
                    .or_default()
                    .entry(i)
                    .or_default()
                    .insert(word.clone());
            }
        }

        Ok(Self { words, database })
    }

    fn randoms(&self) -> RandomWords {
        RandomWords::new(self)
    }

    fn with_letter_at(&self, letter: char, pos: usize) -> Option<HashSetIter<'_, String>> {
        if let Some(sets) = self.database.get(&letter) {
            if let Some(candidates) = sets.get(&pos) {
                return Some(candidates.iter());
            }
        }
        None
    }
}

struct RandomWords<'a> {
    words: &'a Words,
}

impl<'a> RandomWords<'a> {
    fn new(words: &'a Words) -> Self {
        Self { words }
    }
}

impl<'a> Iterator for RandomWords<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let index = random::<usize>() % self.words.words.len();
        Some(self.words.words[index].clone())
    }
}

#[derive(Clone, Copy)]
enum Hint {
    Excluded,
    CorrectPos(char),
    WrongPos(char),
}

impl Hint {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '-' => Some(Hint::Excluded),
            'A'..='Z' => Some(Hint::CorrectPos(c.to_ascii_lowercase())),
            'a'..='z' => Some(Hint::WrongPos(c)),
            _ => None,
        }
    }
}

trait InputRead {
    fn read_game_hints() -> io::Result<[Hint; 5]>;
    fn is_word_in_the_list() -> io::Result<bool>;

    fn get_valid(iter: impl Iterator<Item = String>) -> io::Result<Option<String>> {
        for word in iter {
            println!("Suggested word: {}", word);
            if Self::is_word_in_the_list()? {
                return Ok(Some(word));
            }
        }
        Ok(None)
    }
}

struct InputReader;

impl InputRead for InputReader {
    fn read_game_hints() -> io::Result<[Hint; 5]> {
        let print_instructions = || {
            println!(
                "Please insert a 5 letter valid word hint. \"-\" for excluded letter, upper case for letter in the right position, and lower case for letter in wrong position.");
        };
        // print_instructions();
        loop {
            let mut input = String::new();
            print!("Insert hint: ");
            io::stdout().flush().unwrap();
            match std::io::stdin().read_line(&mut input) {
                Ok(6) => {
                    if let Ok(step) = input
                        .trim()
                        .chars()
                        .filter_map(Hint::from_char)
                        .collect::<Vec<_>>()
                        .try_into()
                    {
                        return Ok(step);
                    } else {
                        print_instructions();
                    }
                }
                _ => print_instructions(),
            }
        }
    }

    fn is_word_in_the_list() -> io::Result<bool> {
        let print_instructions = || {
            print!("Is the word in the list? (y) yes, (n) no: ");
            io::stdout().flush().unwrap();
        };
        print_instructions();
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            match input.as_str().trim() {
                "Y" | "y" => return Ok(true),
                "N" | "n" => return Ok(false),
                _ => print_instructions(),
            }
        }
    }
}

struct Game<I: InputRead> {
    excluded_letters: HashSet<char>,
    phantom: std::marker::PhantomData<I>,
}

impl<I: InputRead> Game<I> {
    fn new() -> Self {
        Self {
            excluded_letters: HashSet::new(),
            phantom: std::marker::PhantomData::default(),
        }
    }

    fn play(&mut self, start: Option<String>) -> io::Result<()> {
        let words = Words::new()?;

        if let Some(start) = &start {
            println!("Starting with {}", start);
        }
        let mut word = match start {
            Some(w) => w,
            None => I::get_valid(words.randoms())?.unwrap(),
        };
        for i in 0..6 {
            let chars: [char; 5] = word.chars().collect::<Vec<char>>().try_into().unwrap();

            let mut hints;
            'get_hints: loop {
                hints = I::read_game_hints()?;

                // Validate input
                for (i, hint) in hints.iter().enumerate() {
                    match hint {
                        Hint::Excluded => (),
                        Hint::CorrectPos(c) | Hint::WrongPos(c) => {
                            if chars[i] != *c {
                                println!(
                                    "Please enter a valid word hint. Some letter currently does not match used word.");
                                continue 'get_hints;
                            }
                        }
                    }
                }
                break;
            }
            if hints.iter().all(|hint| matches!(hint, Hint::CorrectPos(_))) {
                println!("Congratulations! 🎉");
                return Ok(());
            } else if i == 5 {
                println!("I'm sorry I was not good enough 😿");
                return Ok(());
            }

            // Add excluded letters
            for (i, hint) in hints.iter().enumerate() {
                if let Hint::Excluded = hint {
                    self.excluded_letters.insert(chars[i]);
                }
            }

            let remaining_positions = hints
                .iter()
                .enumerate()
                .filter_map(|(pos, hint)| match hint {
                    Hint::WrongPos(_) | Hint::Excluded => Some(pos),
                    _ => None,
                })
                .collect::<Vec<_>>();

            let no_excluded_letters = |word: &&String| {
                let chars = word.chars().collect::<Vec<char>>();
                self.excluded_letters
                    .iter()
                    .all(|letter| remaining_positions.iter().all(|pos| chars[*pos] != *letter))
            };

            let mut candidates = HashSet::<String>::from_iter(
                words.words.iter().filter(no_excluded_letters).cloned(),
            );

            // Get candidates from correct position
            for (i, hint) in hints.iter().enumerate() {
                if let Hint::CorrectPos(c) = hint {
                    if let Some(new_candidates) = words.with_letter_at(*c, i) {
                        candidates = candidates
                            .intersection(
                                &new_candidates
                                    .filter(no_excluded_letters)
                                    .cloned()
                                    .collect::<HashSet<_>>(),
                            )
                            .cloned()
                            .collect();
                    }
                }
            }

            // Get candidates from incorrect position
            let correct_positions = hints
                .iter()
                .enumerate()
                .filter_map(|(pos, hint)| match hint {
                    Hint::CorrectPos(_) => Some(pos),
                    _ => None,
                })
                .collect::<Vec<_>>();

            for (i, hint) in hints.iter().enumerate() {
                if let Hint::WrongPos(c) = hint {
                    candidates = candidates
                        .into_iter()
                        .filter(|word| match word.find(*c) {
                            Some(pos) => pos != i && !correct_positions.contains(&pos),
                            None => false,
                        })
                        .collect();
                }
            }
            println!("Number of candidates: {:?}", candidates.len());

            if let Some(w) = I::get_valid(candidates.into_iter())? {
                word = w;
            } else {
                println!("No more candidates found 😿");
                break;
            }
        }

        println!("I'm sorry I was not good enough 😿");
        Ok(())
    }
}

/// WORDLE solver
///
/// Hints are provided as 5 letter words. "-" means excluded letter.
/// Upper case means letter in the right position. Lower case means letter in wrong position
#[derive(StructOpt, Debug)]
#[structopt(name = "wordle")]
struct Opt {
    #[structopt(short, long)]
    start: Option<String>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    println!("Welcome to the WORDLE solver!");

    let mut game = Game::<InputReader>::new();
    game.play(opt.start)?;

    Ok(())
}
