use std::borrow::Borrow;
use std::num::ParseIntError;
use std::{ops::RangeInclusive, time::Duration};
use std::path::Path;
use std::io;
use std::io::BufRead;
use rand::Rng;
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
    #[snafu(display("No ':' in range {unsplittable_string} (should be x:y)"))]
    Split { unsplittable_string: String },

    #[snafu(display("Too many colons in {unsplittable_string}"))]
    TooManyValues { unsplittable_string: String },

    #[snafu(display("Failed to parse {unparseable_integer_string} into an integer"))]
    Parse { source: ParseIntError, unparseable_integer_string: String }
}

fn parse_range(argument: &str) -> Result<RangeInclusive<usize>, ParseRangeError> {
    let found: Vec<&str> = argument.split(':').collect();
    if found.len() > 2 {
        return Err(ParseRangeError::TooManyValues { unsplittable_string: argument.to_owned() })
    }
    if found.is_empty() {
        return Err(ParseRangeError::Split { unsplittable_string: argument.to_owned() })
    }

    let [start, end] = [found[0], found[1]].map(|s| s.parse::<usize>().with_context(|_| ParseSnafu { unparseable_integer_string: s}));
    Ok(start?..=end?)
}

// Fallible function that tries to return a vector of every line in a file
fn get_words(path: &Path) -> Result<Vec<String>, io::Error> {
    let reader: io::BufReader<std::fs::File> = io::BufReader::new(std::fs::File::open(path)?);
    reader.lines().collect()
}

// Abstraction to make code more readable,
// finds a random line in a file and returns it if it's length
// is within the bounds of the range
fn get_word(word_length: &RangeInclusive<usize>, file_path: &Path ) -> Option<String> {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let words: Vec<String> = get_words(file_path).ok()?;
    loop {
        let word: &String = &words[rng.gen_range(0..words.len())];
        if word_length.contains(&word.len()) {
            return Some(word.to_string());
        }
    };
}

// Simple function to display only the characters in a specified subset, like this:
// if word = "hello", and the list contained ['h', 'l'], it would return
// "h _ l l _ "
fn generate_hangman_word_display(correct_guesses: &[char], word: &str) -> String {
    let mut correct_letter_display = String::new();
        for character in word.chars() {
            if correct_guesses.contains(&character) {
                correct_letter_display.push_str(format!("{} ", character).borrow());
            } else {
                correct_letter_display.push_str("_ ");
            }
        }
    correct_letter_display
}

// Simple function to generate a comma-seperated list of characters
fn generate_list_of_character_display(characters: &[char]) -> String {
    let mut final_string = String::new();
    for guess in characters.iter().enumerate() {
        if guess.0 == 0 {
            final_string.push(*guess.1);
        } else {
            final_string.push_str(format!(", {}", *guess.1).borrow());
        }
    }
    final_string
}

fn validate_input(input: &String, wrong_guesses: &[char], correct_guesses: &[char]) -> Result<(), &'static str>{
    let input = input.to_ascii_lowercase();
    let mut all_good = true;
    let mut message: &str = "Unexpected Error";

    if input.len() != 1 
    {
        all_good = false;
        message = "Input must be one character";
    }

    if wrong_guesses.contains(&input.chars().next().unwrap()) 
    || correct_guesses.contains(&input.chars().next().unwrap()) 
    {
        all_good = false;
        message = "You gave a previously revealed answer, ya dunce!"
    }

    if all_good {
        Ok(())
    } else {
        Err(message)
    }
}

// one-time abstraction to make code more readable
fn suffix(number: usize) -> String {
    match number {
        1 => "".to_owned(),
        _ => "es".to_owned(),
    }
}

fn main() {
    let args = Args::parse();

    // Pick a random word within the bounds of the allowed word length from said words file
    let word = if let Some(word) = get_word(&args.word_length_range, Path::new(&args.filename)) {
        word
    } else {
        println!("File {} doesn't exist or doesn't contain words matching the length criteria of {} to {}", args.filename, args.word_length_range.start(), args.word_length_range.end());
        return
    };

    let mut wrong_guesses: Vec<char> = Vec::<char>::new();
    let mut correct_guesses: Vec<char> = Vec::<char>::new();

    clear().expect("failed to clear screen");

    // Game loop
    loop {

        let state: GameState;
        if correct_guesses.len() == word.len() {
            state = GameState::Won;
        } else if wrong_guesses.len() == args.wrong_guesses_allowed {
            state = GameState::Lost;
        } else {
            state = GameState::Playing;
        }

        let wrong_guesses_remaining = args.wrong_guesses_allowed - wrong_guesses.len();
        match state {
            GameState::Playing => {
                println!("{}\n", generate_hangman_word_display(&correct_guesses, &word));

                if !wrong_guesses.is_empty() {

                    if wrong_guesses_remaining > 3 {
                        println!("{} Incorrect Guess{} Remaining.\n", wrong_guesses_remaining, suffix(wrong_guesses_remaining));
                    } else {
                        println!("Only {} Incorrect Guess{} Left!\n", wrong_guesses_remaining, suffix(wrong_guesses_remaining));
                    }

                    println!("Wrong Guesses: {}\n", generate_list_of_character_display(&wrong_guesses));
                }
            }
            
            GameState::Won => {
                println!("{}\n", word.chars().fold(String::new(), |mut output, character| {let _ = write!(output, "{} ", character); output}));
                println!("You guessed the word\n{} incorrect guesses, with {} wrong guess{} remaining", wrong_guesses.len(), wrong_guesses_remaining, suffix(wrong_guesses_remaining));

                sleep(Duration::from_secs(1));
                return
            }

            GameState::Lost => {
                println!("You failed, the word was {}\n", word);
                sleep(Duration::from_secs(1));
                return
            }
        }

        let guess: char = Input::<String>::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Enter a guess")
                                        .validate_with(|s: &String| validate_input(s, &wrong_guesses, &correct_guesses))
                                        .interact_text()
                                        .unwrap()
                                        .chars().next().unwrap()
                                        .to_ascii_lowercase();
        
        let num_occurances = word.matches(guess).count();
        if num_occurances == 0 {
            wrong_guesses.push(guess);
            println!("Wrong!");
            sleep(Duration::from_secs(1));
        } else {
            for _ in 0..num_occurances {
                correct_guesses.push(guess);
            }
            println!("Correct!");
            sleep(Duration::from_secs(1));
        }

        clear().expect("failed to clear screen");
    }
}
