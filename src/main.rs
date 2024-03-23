use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use console::{Style, Term};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;

fn read_pipe() -> String {
    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional

    let mut line = String::new();

    // Could also `match` on the `Result` if you wanted to handle `Err`
    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }
    }

    line
}

fn random<R>(rng: &mut R, words: &Vec<&str>, n: usize) -> String
where
    R: Rng + ?Sized,
{
    words
        .choose_multiple(rng, n)
        .cloned()
        .intersperse(" ")
        .collect()
}

fn permutate<R>(
    rng: &mut R,
    mut words: Vec<&str>,
    combination: usize,
    repetition: usize,
) -> Vec<String>
where
    R: Rng + ?Sized,
{
    words.shuffle(rng);

    words
        .iter()
        .chunks(combination)
        .into_iter()
        .map(|mut chunk| {
            let mut test = chunk.join(" ");
            test.push(' ');
            test = test.repeat(repetition);
            test.pop();
            test
        })
        .collect()
}

struct TestResult {
    test: String,
    keys: Vec<(Instant, char)>,
    correct: usize,
    incorrect: usize,
}

impl TestResult {
    fn len(&self) -> Duration {
        if self.keys.len() == 0 {
            Duration::new(0, 0)
        } else {
            self.keys.last().unwrap().0 - self.keys.first().unwrap().0
        }
    }

    fn cpm(&self) -> f64 {
        self.keys.len() as f64 / (self.len().as_secs_f64() / 60.0)
    }

    fn wpm(&self) -> f64 {
        self.cpm() / 5.0
    }

    fn accuracy(&self) -> f64 {
        self.correct as f64 / self.keys.len() as f64
    }
}

fn run_test(test: String) -> io::Result<TestResult> {
    let test_style = Style::new().magenta();
    let correct_style = Style::new();
    let incorrect_style = Style::new().red().underlined();

    let mut term = Term::stdout();
    write!(term, "{}", test_style.apply_to(test.clone()))?;
    term.move_cursor_left(test.len())?;

    let mut i = 0;
    let mut keys: Vec<(Instant, char)> = vec![];
    let mut correct = 0;
    let mut incorrect = 0;

    let interrupt_error = io::Error::new(io::ErrorKind::Interrupted, "canceled");

    loop {
        match term.read_key()? {
            console::Key::Escape | console::Key::CtrlC => {
                return Err(interrupt_error);
            }
            console::Key::Backspace => {
                if i > 0 {
                    i -= 1;
                    term.move_cursor_left(1)?;
                    write!(
                        term,
                        "{}",
                        test_style.apply_to(test.chars().nth(i).unwrap())
                    )?;
                    term.move_cursor_left(1)?;
                }
            }
            console::Key::Char(c) => {
                keys.push((Instant::now(), c));
                if c == test.chars().nth(i).unwrap() {
                    write!(term, "{}", correct_style.apply_to(c))?;
                    correct += 1;
                } else {
                    write!(term, "{}", incorrect_style.apply_to(c))?;
                    incorrect += 1;
                }
                i += 1;
                if i == test.len() {
                    break;
                }
            }
            _ => {}
        }
    }

    write!(term, "\n\n")?;

    Ok(TestResult {
        test,
        keys,
        correct,
        incorrect,
    })
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'w', long)]
    min_wpm: Option<f64>,
    #[arg(short = 'a', long)]
    min_accuracy: Option<f64>,

    #[command(subcommand)]
    generate: GeneratorArgs,
}

#[derive(Subcommand, Debug)]
enum GeneratorArgs {
    Random {
        words: usize,
    },
    Permutation {
        #[arg(short, long)]
        combination: usize,
        #[arg(short, long)]
        repetition: usize,
    },
}

fn main() -> io::Result<()> {
    let cli = Args::parse();

    let mut rng = rand::thread_rng();
    let set_string = read_pipe();
    let set = set_string
        .split('\n')
        .filter(|line| line.len() > 0)
        .collect_vec();

    if set.len() <= 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Input stream doesn't contain words",
        ));
    }

    match cli.generate {
        GeneratorArgs::Random { words } => loop {
            let test = random(&mut rng, &set, words);

            let mut result = run_test(test)?;
            print_result(&result);

            while cli.min_wpm.map_or(false, |min| result.wpm() < min)
                || cli
                    .min_accuracy
                    .map_or(false, |min| result.accuracy() < min / 100.0)
            {
                result = run_test(result.test)?;
                print_result(&result);
            }
        },
        GeneratorArgs::Permutation {
            combination,
            repetition,
        } => {
            let permutations = permutate(&mut rng, set.clone(), combination, repetition);
            let len = permutations.len();
            for (i, test) in permutations.into_iter().enumerate() {
                println!("{} / {}", i + 1, len);
                let mut result = run_test(test)?;
                print_result(&result);

                while cli.min_wpm.map_or(false, |min| result.wpm() < min)
                    || cli
                        .min_accuracy
                        .map_or(false, |min| result.accuracy() < min / 100.0)
                {
                    result = run_test(result.test)?;
                    print_result(&result);
                }
            }
        }
    }

    Ok(())
}

fn print_result(result: &TestResult) {
    println!("wpm: {:.2}", result.wpm());
    println!("accuracy: {:.2}%", result.accuracy() * 100.0);
    println!();
}
