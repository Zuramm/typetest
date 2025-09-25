pub mod generator;

use std::time::{Duration, Instant};

use itertools::Itertools;

pub enum Input {
    Character(char),
    DeleteLetter,
    DeleteWord,
}

#[derive(Debug, PartialEq)]
pub enum InputKind {
    Correct,
    Incorrect,
    Additional,
    Missed,
}

pub struct InputResult {
    pub kind: Option<InputKind>,
    pub is_done: bool,
}

pub struct RunningTest {
    pub expected: Vec<char>,
    pub cursor: usize,
    inputs: Vec<(Instant, Input)>,
    correct: usize,
    incorrect: usize,
}

fn correct_input_naive(expected: &str, inputs: &[Input]) -> Vec<InputKind> {
    let chars = expected.chars().collect_vec();
    let mut cursor = 0usize;
    let mut kinds = Vec::<InputKind>::new();
    let mut spaces = Vec::<usize>::new();

    println!("cursor: {cursor}");
    for input in inputs {
        match input {
            Input::Character(c) => {
                if let Some(expected_c) = chars.get(cursor) {
                    if expected_c == c {
                        kinds.push(InputKind::Correct);
                    } else {
                        kinds.push(InputKind::Incorrect);
                    }
                } else {
                    kinds.push(InputKind::Additional);
                }
                if c.is_whitespace() {
                    spaces.push(cursor);
                }
                cursor += 1;
                println!("character {c}: {cursor}");
            }
            Input::DeleteLetter => {
                if let Some(&last) = spaces.last() {
                    if last == cursor {
                        spaces.pop();
                    }
                }
                if cursor > 0 {
                    cursor -= 1;
                }
                println!("delete letter: {cursor}");
            }
            Input::DeleteWord => {
                println!("spaces: {spaces:?}");
                let mut word_start = 0;
                while cursor > 0 {
                    cursor -= 1;
                    word_start = spaces.pop().map_or(0, |s| s + 1);
                    println!("{word_start} < {cursor}");
                    if word_start <= cursor {
                        break;
                    }
                }
                cursor = word_start;
                println!("delete word: {cursor}");
            }
        }
    }

    for _ in cursor..expected.len() {
        kinds.push(InputKind::Missed);
    }

    kinds
}

impl RunningTest {
    pub fn new(expected: &str) -> Self {
        Self {
            expected: expected.chars().collect(),
            inputs: vec![],
            cursor: 0,
            correct: 0,
            incorrect: 0,
        }
    }

    pub fn input(&mut self, time: Instant, input: Input) -> InputResult {
        let kind: Option<InputKind>;
        let is_done: bool;
        match input {
            Input::Character(c) => {
                if c == self.expected[self.cursor] {
                    kind = Some(InputKind::Correct);
                    self.correct += 1;
                } else {
                    kind = Some(InputKind::Incorrect);
                    self.incorrect += 1;
                }
                self.cursor += 1;
                is_done = self.cursor == self.expected.len();
            }
            Input::DeleteLetter => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                kind = None;
                is_done = false;
            }
            Input::DeleteWord => todo!(),
        }

        self.inputs.push((time, input));
        InputResult { kind, is_done }
    }
}

impl From<RunningTest> for TestResult {
    fn from(value: RunningTest) -> Self {
        TestResult {
            inputs: value.inputs,
            correct: value.correct,
            incorrect: value.incorrect,
        }
    }
}

pub struct TestResult {
    inputs: Vec<(Instant, Input)>,
    correct: usize,
    incorrect: usize,
}

impl TestResult {
    fn len(&self) -> Duration {
        if let (Some(&(start, _)), Some(&(end, _))) = (self.inputs.first(), self.inputs.last()) {
            end - start
        } else {
            Duration::new(0, 0)
        }
    }

    fn cpm(&self) -> f64 {
        self.inputs.len() as f64 / (self.len().as_secs_f64() / 60.0)
    }

    pub fn wpm(&self) -> f64 {
        self.cpm() / 5.0
    }

    pub fn accuracy(&self) -> f64 {
        self.correct as f64 / (self.correct + self.incorrect) as f64
    }

    fn keypresses_each_second(&self) -> Vec<usize> {
        if let Some(first) = self.inputs.first() {
            let mut history = Vec::<usize>::new();
            let mut start = first.0;
            let mut count = 0;

            for (time, _char) in self.inputs.iter().skip(1) {
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
        } else {
            Vec::new()
        }
    }

    pub fn consistency(&self) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! inputs {
        (@single $c:literal) => {
            inputs!(@literal $c)
        };
        (@single delete_letter) => { vec![Input::DeleteLetter] };
        (@single delete_word) => { vec![Input::DeleteWord] };

        (@literal $s:expr) => {{
            let s: &str = $s;
            s.chars().map(Input::Character).collect::<Vec<_>>()
        }};

        () => {
            Vec::<Input>::new()
        };

        ($($input:tt),* $(,)?) => {{
            let mut result = Vec::new();
            $(
                result.extend(inputs!(@single $input));
            )*
            result
        }};
    }

    macro_rules! correction {
        () => {
            Vec::<InputKind>::new()
        };

        ($($symbol:tt)*) => {
            vec![
                $(
                    match stringify!($symbol) {
                        "&" => InputKind::Incorrect,
                        "+" => InputKind::Additional,
                        "-" => InputKind::Missed,
                        _ => InputKind::Correct,
                    }
                ),*
            ]
        };
    }

    mod correct_input_naive {
        use super::*;

        macro_rules! test_cases  {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected_text, inputs, expected_kinds) = $value;
                    assert_eq!(expected_kinds, correct_input_naive(expected_text, &inputs));
                }
            )*
            }
        }

        test_cases! {
            empty_input: (
                "hello",
                inputs![],
                correction![- - - - -]
            ),

            correct_simple: (
                "hello",
                inputs!["hello"],
                correction![h e l l o]
            ),

            incorrect_simple: (
                "hello",
                inputs!["hxlyo"],
                correction![h & l & o]
            ),

            additional_simple: (
                "hello",
                inputs!["hellor"],
                correction![h e l l o +]
            ),

            with_delete_letter: (
                "hello",
                inputs!["hx", delete_letter, "el"],
                correction![h & e l - -]
            ),

            empty_input_with_delete_letter: (
                "",
                inputs![delete_letter],
                correction![]
            ),

            with_delete_word: (
                "hello world",
                inputs!["hello wx", delete_word, "wo"],
                correction![h e l l o _ w & w o - - -]
            ),

            empty_input_with_delete_word: (
                "",
                inputs![delete_word],
                correction![]
            ),

            with_delete_word_with_double_space_before: (
                "hello world",
                inputs!["hello  w", delete_word, delete_letter, "wo"],
                correction![h e l l o _ & & w o - - -]
            ),

            with_delete_word_with_double_space_after: (
                "hello",
                inputs!["hello  ", delete_word],
                correction![h e l l o + + - - - - -]
            ),

            delete_word_from_beginning: (
                "test word",
                inputs!["tesx", delete_word, "test"],
                correction![t e s & t e s t - - - - -]
            ),
        }
    }
}
