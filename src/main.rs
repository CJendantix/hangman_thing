use std::{ops::RangeInclusive, time::Duration};
use std::path::PathBuf;
use std::io;
use std::io::BufRead;
use rand::Rng;
use dialoguer::{theme::ColorfulTheme, Input};
use clearscreen::clear;
use std::thread::sleep;

// Game settings
const NUM_WRONG_GUESSES: usize = 8;

const RANGE_WORD_LENGTH_ALLOWED: RangeInclusive<usize> = 3..=8;

fn get_words(path: PathBuf) -> Result<Vec<String>, io::Error> {
    let reader: io::BufReader<std::fs::File> = io::BufReader::new(std::fs::File::open(path)?);
    reader.lines().collect()
}

fn find_word(range: &RangeInclusive<usize>, path: PathBuf ) -> Option<String> {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let words: Vec<String> = get_words(path).ok()?;
    loop {
        let word: &String = &words[rng.gen_range(0..words.len())];
        if range.contains(&word.len()) {
            return Some(word.to_string());
        }
    };
}

fn generate_hangman_word_display(correct_guesses: &[char], word: &str) -> String {
    let mut correct_letter_display = String::new();
        for character in word.chars() {
            if correct_guesses.contains(&character) {
                correct_letter_display.push(character);
                correct_letter_display.push(' ');
            } else {
                correct_letter_display.push('_');
                correct_letter_display.push(' ');
            }
        }
    correct_letter_display
}

fn generate_list_of_character_display(characters: &[char]) -> String {
    let mut final_string = String::new();
    for guess in characters.iter().enumerate() {
        if guess.0 == 0 {
            final_string.push(*guess.1);
        } else {
            final_string.push(',');
            final_string.push(' ');
            final_string.push(*guess.1);
        }
    }
    final_string
}

fn main() {
    let word = find_word(&RANGE_WORD_LENGTH_ALLOWED, PathBuf::from("./words_alpha.txt")).expect("File doesn't exist or is empty!");
    let mut wrong_guesses: Vec<char> = Vec::<char>::new();
    let mut correct_guesses: Vec<char> = Vec::<char>::new();
    clear().expect("failed to clear screen");

    loop {
        

        println!("{}\n", generate_hangman_word_display(&correct_guesses, &word));
        let wrong_guesses_remaining = NUM_WRONG_GUESSES - wrong_guesses.len();
        if !wrong_guesses.is_empty() && wrong_guesses.len() != NUM_WRONG_GUESSES {
            println!("Only {} Incorrect Guess{} Left!\n", wrong_guesses_remaining, {
                match wrong_guesses_remaining {
                    1 => "",
                    _ => "es",
                }
            });
            println!("Wrong Guesses: {}\n", generate_list_of_character_display(&wrong_guesses));
        }

        if correct_guesses.len() == word.len() {
            println!("You guessed the word!");
            return
        }
        if wrong_guesses.len() == NUM_WRONG_GUESSES {
            println!("You failed, the word was {}\n", word);
            return
        }

        // Logic & Boilerplate to take specific character input, ignore.
        let guess: char = Input::<String>::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Enter a guess")
                                        .validate_with(|input: &String| -> Result<(), &str> {
                                            let input = input.to_ascii_lowercase();
                                            let mut all_good = true;
                                            let mut message: &str = "Unexpected Error";
                                            if input.len() != 1 {
                                                all_good = false;
                                                message = "Input must be one character";
                                            } else if wrong_guesses.contains(&input.chars().collect::<Vec<char>>()[0]) || correct_guesses.contains(&input.chars().collect::<Vec<char>>()[0]){
                                                all_good = false;
                                                message = "You gave a previously revealed answer, ya dunce!"
                                            }
                                            if all_good {Ok(())} else {
                                                Err(message)
                                            }
                                        })
                                        .interact_text()
                                        .unwrap()
                                        .chars().collect::<Vec<char>>()[0]
                                        .to_ascii_lowercase();
    
        let mut num_occurances = 0;
        for character in word.chars() {
            if character == guess {num_occurances += 1;}
        }
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