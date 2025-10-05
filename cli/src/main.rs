use std::io::{self, Read, Write};
use std::time::Instant;

use clap::{command, Parser, Subcommand};
use console::{Style, Term};
use itertools::Itertools;
use logic::{correct_input_naive, generator, output_text, Input, InputKind, InputText};

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

#[derive(Default)]
struct TestResult {
    timestamps: Vec<Instant>,
    inputs: Vec<Input>,
}

impl TestResult {
    fn character(&mut self, c: char) {
        self.timestamps.push(Instant::now());
        self.inputs.push(Input::Character(c));
    }

    fn delete_letter(&mut self) {
        self.timestamps.push(Instant::now());
        self.inputs.push(Input::DeleteLetter);
    }
}

fn run_test(test: &str) -> io::Result<TestResult> {
    let test_style = Style::new().magenta();
    let correct_style = Style::new();
    let incorrect_style = Style::new().red().underlined();

    let mut term = Term::stdout();
    write!(term, "{}", test_style.apply_to(test))?;
    term.move_cursor_left(test.len())?;

    let mut cursor = 0;
    let mut result = TestResult::default();

    let interrupt_error = io::Error::new(io::ErrorKind::Interrupted, "canceled");

    loop {
        let mut result_changed = false;
        match term.read_key()? {
            console::Key::Escape | console::Key::CtrlC => {
                return Err(interrupt_error);
            }
            console::Key::Backspace => {
                result.delete_letter();
                result_changed = true;
            }
            console::Key::Char(c) => {
                result.character(c);
                result_changed = true;
            }
            _ => {}
        }
        if result_changed {
            let (positions, corrections, typed, is_done) =
                correct_input_naive(test, &result.inputs);
            term.move_cursor_left(cursor);
            for (kind, text) in output_text(&positions, &corrections, &typed) {
                let style = match kind {
                    InputKind::Correct => &correct_style,
                    InputKind::Incorrect => &incorrect_style,
                    InputKind::Additional => &incorrect_style,
                    InputKind::Missed => &test_style,
                };
                write!(term, "{}", style.apply_to(text));
            }
            if typed.len() < cursor {
                write!(term, "{}", test_style.apply_to(&test[typed.len()..cursor]));
                term.move_cursor_left(cursor - typed.len());
            }
            cursor = typed.len();
            if is_done {
                break;
            }
        }
    }

    write!(term, "\n\n")?;

    Ok(result)
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
        .filter(|line| !line.is_empty())
        .collect_vec();

    if set.len() <= 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Input stream doesn't contain words",
        ));
    }

    let min_wpm = cli.min_wpm.unwrap_or(0.);
    let min_accuracy = cli.min_accuracy.unwrap_or(0.);
    let min_consistency = cli.min_consistency.unwrap_or(0.);

    match cli.generate {
        GeneratorArgs::Random { words } => {
            let mut all_results = Vec::<TestResult>::new();
            let test = generator::random(&mut rng, &set, words);

            run_test_with_requirement(
                &test,
                min_wpm,
                min_accuracy,
                min_consistency,
                &mut all_results,
            )?;

            print_final_result(all_results);
        }
        GeneratorArgs::Permutation {
            combination,
            repetition,
        } => {
            let mut all_results = Vec::<TestResult>::new();
            let permutations = generator::permutate(&mut rng, set.clone(), combination, repetition);
            let len = permutations.len();
            for (i, test) in permutations.iter().enumerate() {
                println!("{} / {}", i + 1, len);
                run_test_with_requirement(
                    test,
                    min_wpm,
                    min_accuracy,
                    min_consistency,
                    &mut all_results,
                )?;
            }

            print_final_result(all_results);
        }
    }

    Ok(())
}

fn run_test_with_requirement(
    test: &str,
    min_wpm: f64,
    min_accuracy: f64,
    min_consistency: f64,
    all_results: &mut Vec<TestResult>,
) -> Result<(), io::Error> {
    let mut result = run_test(test)?;
    print_result(&result);
    while result.wpm() < min_wpm
        || result.accuracy() < min_accuracy / 100.0
        || result.consistency() < min_consistency / 100.0
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

fn print_final_result(all_results: Vec<TestResult>) {
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
