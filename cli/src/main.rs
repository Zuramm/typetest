use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use clap::{command, Parser, Subcommand};
use console::{Style, Term};
use itertools::Itertools;
use rand::seq::{IndexedRandom, SliceRandom};
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

struct TestResult<'a> {
    test: &'a str,
    keys: Vec<(Instant, char)>,
    correct: usize,
    incorrect: usize,
}

impl<'a> TestResult<'a> {
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

    fn keypresses_each_second(&self) -> Vec<usize> {
        match self.keys.first() {
            Some(first) => {
                let mut history = Vec::<usize>::new();
                let mut start = first.0;
                let mut count = 0;

                for (time, _char) in self.keys.iter().skip(1) {
                    let time = *time;
                    if (time - start).as_secs_f64() > 1.0 {
                        history.push(count);
                        count = 0;
                        start = time;
                    } else {
                        count += 1;
                    }
                }

                history.push(count);

                history
            }
            None => Vec::new(),
        }
    }

    fn consistency(&self) -> f64 {
        let history = self
            .keypresses_each_second()
            .iter()
            .map(|count| (*count as f64) / 5.0)
            .collect_vec();
        let mean = history.iter().sum::<f64>() / (history.len() as f64);
        let standard_deviation = history
            .iter()
            .map(|count| (*count - mean).powi(2))
            .sum::<f64>()
            / (history.len() as f64);

        standard_deviation / mean
    }
}

fn run_test<'a>(test: &'a str) -> io::Result<TestResult<'a>> {
    let test_style = Style::new().magenta();
    let correct_style = Style::new();
    let incorrect_style = Style::new().red().underlined();

    let mut term = Term::stdout();
    write!(term, "{}", test_style.apply_to(test))?;
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
    /// Only proceed to the next test if the words per minute are at least the provided amount.
    #[arg(short = 'w', long)]
    min_wpm: Option<f64>,
    /// Only proceed to the next test if the accuracy is at least the provided amount.
    /// Values go from 0 to 100.
    #[arg(short = 'a', long)]
    min_accuracy: Option<f64>,
    /// Only proceed to the next test if the consistency is at least the provided amount.
    /// Values go from 0 to 100.
    #[arg(short = 'c', long)]
    min_consistency: Option<f64>,

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

    let mut rng = rand::rng();
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
            let mut all_results = Vec::<TestResult>::new();
            let test = random(&mut rng, &set, words);

            run_test_with_requirement(&test, &cli, &mut all_results)?;

            print_final_result(all_results);
        },
        GeneratorArgs::Permutation {
            combination,
            repetition,
        } => {
            let mut all_results = Vec::<TestResult>::new();
            let permutations = permutate(&mut rng, set.clone(), combination, repetition);
            let len = permutations.len();
            for (i, test) in permutations.iter().enumerate() {
                println!("{} / {}", i + 1, len);
                run_test_with_requirement(test, &cli, &mut all_results)?;
            }

            print_final_result(all_results);
        }
    }

    Ok(())
}

fn run_test_with_requirement<'a, 'b>(
    test: &'a String,
    cli: &'b Args,
    all_results: &mut Vec<TestResult<'a>>,
) -> Result<(), io::Error> {
    let mut result = run_test(test)?;
    print_result(&result);
    while cli.min_wpm.map_or(false, |min| result.wpm() < min)
        || cli
            .min_accuracy
            .map_or(false, |min| result.accuracy() < min / 100.0)
        || cli
            .min_consistency
            .map_or(false, |min| result.consistency() < min / 100.0)
    {
        all_results.push(result);
        result = run_test(test)?;
        print_result(&result);
    }
    all_results.push(result);
    Ok(())
}

fn kogasa(value: f64) -> f64 {
    100.0 * (1.0 - (value + value.powi(3) / 3.0 + value.powi(5) / 5.0).tanh())
}

fn print_final_result(all_results: Vec<TestResult<'_>>) {
    println!("total tests: {}", all_results.len());
    println!(
        "average wpm: {:.2}",
        all_results.iter().map(|result| result.wpm()).sum::<f64>() / all_results.len() as f64
    );
    println!(
        "average accuracy: {:.2}%",
        all_results
            .iter()
            .map(|result| result.accuracy())
            .sum::<f64>()
            * 100.0
            / all_results.len() as f64
    );
    println!(
        "average consistency: {:.2}%",
        all_results
            .iter()
            .map(|result| kogasa(result.consistency()))
            .sum::<f64>()
            / all_results.len() as f64
    );
}

fn print_result(result: &TestResult) {
    println!("wpm: {:.2}", result.wpm());
    println!("accuracy: {:.2}%", result.accuracy() * 100.0);
    println!("consistency: {:.2}%", kogasa(result.consistency()));
    println!();
}
