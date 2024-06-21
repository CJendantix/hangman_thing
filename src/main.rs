use std::num::ParseIntError;
use std::{ops::RangeInclusive, time::Duration};
use std::path::{Path, PathBuf};
use std::io;
use std::io::BufRead;
use rand::prelude::SliceRandom;
use dialoguer::{theme::ColorfulTheme, Input};
use clearscreen::clear;
use snafu::{ResultExt, Snafu};
use std::thread::sleep;
use std::fmt::Write;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short = 'f', long = "filename", default_value = "./words.txt") ]
    filename: String,

    #[arg(short = 'g', long = "wrong_guesses", default_value_t = 8)]
    wrong_guesses_allowed: usize,

    #[arg(short = 'l', long = "word_length", default_value = "3:8", value_parser = parse_range)]
    word_length_range: RangeInclusive<usize>,
}

enum GameState {
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Snafu)]
enum ParseRangeError {
    #[snafu(display("Invalid Format '{value}' (should be x:y)"))]
    Format { value: String },

    #[snafu(display("Failed to parse '{value}' into an integer"))]
    Parse { source: ParseIntError, value: String },

    #[snafu(display("The first range bound cannot be larger than the second '{}:{}'", value[0], value[1]))]
    StartOutOfBounds { value: [usize; 2] },
}

fn parse_range(argument: &str) -> Result<RangeInclusive<usize>, ParseRangeError> {
    let found: Vec<&str> = argument.split(':').collect();
    if found.len() != 2
    || found.iter().copied().any(|s| s.is_empty()) {
        return Err(ParseRangeError::Format { value: argument.to_owned() })
    }

    let [start, end] = [found[0], found[1]]
    .map(|s| s.parse::<usize>().with_context(|_| ParseSnafu { value: s}));
    let [start, end] = [start?, end?];

    if start > end {
        return Err(ParseRangeError::StartOutOfBounds { value: [start, end] })
    }
    
    Ok(start..=end)
}

#[derive(Debug, Snafu)]
enum GetWordsError {
    #[snafu(display("Problem Opening the words list, try running with --help!"))]
    Unexpected { source: io::Error, unopenable_file: PathBuf },

    #[snafu(display("Words List is empty!"))]
    Empty { empty_file: PathBuf }
}

// Fallible function that tries to return a vector of every line in a file
fn get_words_list(path: &Path) -> Result<Vec<String>, GetWordsError> {
    let lines: Result<Vec<String>, io::Error> = io::BufReader::new(std::fs::File::open(path).with_context(|_| UnexpectedSnafu { unopenable_file: path })?)
        .lines()
        .collect();

    let lines = lines.with_context(|_| UnexpectedSnafu { unopenable_file: path })?;

    if lines.is_empty() {
        return Err(GetWordsError::Empty { empty_file: PathBuf::from(path) });
    }

    Ok(lines)
}

// Abstraction to make code more readable,
// finds a random line in a file and returns it if it's length
// is within the bounds of the range
fn get_word(word_length: &RangeInclusive<usize>, path: &Path ) -> Result<String, GetWordsError> {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let words: Vec<String> = get_words_list(path)?
        .into_iter()
        .filter(|word| word_length.contains(&word.len()))
        .collect();
    
    if words.is_empty() {
        return Err(GetWordsError::Empty { empty_file: PathBuf::from(path) });
    }

    Ok(words.choose(&mut rng).unwrap().to_string())
}

// Simple function to display only the characters in a specified subset, like this:
// if word = "hello", and the list contained ['h', 'l'], it would return
// "h _ l l _ "
fn handman_display(correct_guesses: &[char], word: &str) -> String {
    let mut correct_letter_display = String::new();
        for character in word.chars() {
            if correct_guesses.contains(&character) {
                correct_letter_display.push_str(format!("{} ", character).as_str());
            } else {
                correct_letter_display.push_str("_ ");
            }
        }
    correct_letter_display
}

// Simple function to generate a comma-seperated list of characters
fn character_list_display(characters: &[char]) -> String {
    let mut final_string = String::new();
    for guess in characters.iter().enumerate() {
        if guess.0 == 0 {
            final_string.push(*guess.1);
        } else {
            final_string.push_str(format!(", {}", *guess.1).as_str());
        }
    }
    final_string
}

fn validate_input(input: &str, wrong_guesses: &[char], correct_guesses: &[char]) -> Result<(), &'static str>{
    let input = input.to_lowercase();
    let character = input.chars().next().unwrap();
    let mut message = "";

    if input.len() != 1 {
        message = "Input must be one character"
    }
    else if wrong_guesses.contains(&character) || correct_guesses.contains(&character)
    {
        message = "You gave a previously revealed answer, ya dunce!"
    }

    if !message.is_empty() {
        return Err(message);
    }

    Ok(())
}

// one-time abstraction to make code more readable
fn suffix(number: usize) -> String {
    match number {
        1 => "".to_owned(),
        _ => "es".to_owned(),
    }
}

#[snafu::report]
fn main() -> Result<(), GetWordsError> {
    let args = Args::parse();

    // Pick a random word within the bounds of the allowed word length from said words file
    let word = get_word(&args.word_length_range, Path::new(&args.filename))?;

    let mut wrong_guesses: Vec<char> = Vec::<char>::new();
    let mut correct_guesses: Vec<char> = Vec::<char>::new();

    clear().expect("failed to clear screen");

    // Game loop
    loop {
        use GameState as GS;
        let state: GS = if correct_guesses.len() >= word.len() {
            GS::Won
        } else if wrong_guesses.len() >= args.wrong_guesses_allowed {
            GS::Lost
        } else {
            GS::Playing
        };

        let wrong_guesses_remaining = args.wrong_guesses_allowed - wrong_guesses.len();
        match state {
            GameState::Playing => {
                println!("{}\n", handman_display(&correct_guesses, &word));

                if !wrong_guesses.is_empty() {

                    if wrong_guesses_remaining > 3 {
                        println!("{} Incorrect Guess{} Remaining.\n", wrong_guesses_remaining, suffix(wrong_guesses_remaining));
                    } else {
                        println!("Only {} Incorrect Guess{} Left!\n", wrong_guesses_remaining, suffix(wrong_guesses_remaining));
                    }

                    println!("Wrong Guesses: {}\n", character_list_display(&wrong_guesses));
                }
            }
            
            GS::Won => {
                println!("{}\n", word.chars().fold(String::new(), |mut output, character| {let _ = write!(output, "{} ", character); output}));
                println!("You guessed the word\n{} incorrect guesses, with {} wrong guess{} remaining", wrong_guesses.len(), wrong_guesses_remaining, suffix(wrong_guesses_remaining));

                sleep(Duration::from_secs(1));
                return Ok(())
            }

            GS::Lost => {
                println!("You failed, the word was {}\n", word);
                sleep(Duration::from_secs(1));
                return Ok(())
            }
        }

        let guess = Input::<String>::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Enter a guess")
                                        .validate_with(|s: &String| validate_input(s, &wrong_guesses, &correct_guesses))
                                        .interact_text()
                                        .unwrap()
                                        .to_lowercase()
                                        .chars().next()
                                        .unwrap();
        if word.contains(guess) {
            println!("Correct");
            correct_guesses.push(guess);
        } else {
            println!("Incorrect");
            wrong_guesses.push(guess);
        }

        sleep(Duration::from_secs(1));

        clear().expect("failed to clear screen");
    }
}
