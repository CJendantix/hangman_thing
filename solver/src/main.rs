use std::io::BufRead;
use std::io;
use std::path::Path;
use regex::Regex;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short = 'f', long = "filename", default_value = "./words.txt") ]
    filename: String,

    #[arg(short = 'i', long = "current_info")]
    info: String,

    #[arg(short = 'l', long = "incorrect_letters", default_value_t = String::from(""))]
    incorrect_letters: String,
}

fn get_words(path: &Path) -> Result<Vec<String>, io::Error> {
    let reader: io::BufReader<std::fs::File> = io::BufReader::new(std::fs::File::open(path)?);
    reader.lines().collect()
}

fn main() {
    let args = Args::parse();
    let correct_letters: String = args.info.chars().filter(|c| *c != '_').collect();
    let re = {
        if !correct_letters.is_empty() {
            Regex::new( 
                args.info.chars().map(|c| 
                if c == '_' {
                    format!("[^{}]", correct_letters)
                } else {
                    c.to_string()
                }).collect::<String>().as_str()
            )
            .unwrap()
        } else {
            Regex::new(args.info.chars().map(|_| '.').collect::<String>().as_str()).unwrap()
        }
    };
    let incorrect_letters: Vec<&str> = if !args.incorrect_letters.is_empty() {args.incorrect_letters.split(',').collect()} else {vec![]};

    let mut words = get_words(Path::new(&args.filename)).unwrap()
                            .into_iter()
                            .filter(|word| 
                                word.len() == args.info.len()
                                && !incorrect_letters.iter().any(|character| word.contains(*character))
                                && re.is_match(word)
                            )
                            .collect::<Vec<String>>();

    words.sort_by_key(|a| a.to_lowercase());

    for word in words {println!("{}", word)}

}
