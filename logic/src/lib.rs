pub mod generator;

use std::time::{Duration, Instant};

use itertools::Itertools;

#[derive(Debug, PartialEq)]
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

pub struct Cursor {
    position: usize,
    spaces: Vec<usize>,
}

impl Cursor {
    fn new() -> Self {
        Self {
            position: 0,
            spaces: vec![],
        }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn input(&mut self, c: char) -> usize {
        if c.is_whitespace() {
            self.spaces.push(self.position);
        }
        self.position += 1;
        self.position
    }

    fn delete_letter(&mut self) -> usize {
        if let Some(&last) = self.spaces.last() {
            if last == self.position {
                self.spaces.pop();
            }
        }
        if self.position > 0 {
            self.position -= 1;
        }
        self.position
    }

    fn delete_word(&mut self) -> usize {
        let mut word_start = 0;
        while self.position > 0 {
            self.position -= 1;
            word_start = self.spaces.pop().map_or(0, |s| s + 1);
            if word_start <= self.position {
                break;
            }
        }
        self.position = word_start;
        self.position
    }
}

pub fn correct_input_naive(expected: &str, inputs: &[Input]) -> (Vec<InputKind>, usize, bool) {
    let mut cursor = Cursor::new();
    let chars = expected.chars().collect_vec();
    let mut kinds = Vec::<InputKind>::new();

    for input in inputs {
        match input {
            Input::Character(c) => {
                if let Some(expected_c) = chars.get(cursor.position()) {
                    if expected_c == c {
                        kinds.push(InputKind::Correct);
                    } else {
                        kinds.push(InputKind::Incorrect);
                    }
                } else {
                    kinds.push(InputKind::Additional);
                }
                cursor.input(*c);
            }
            Input::DeleteLetter => {
                cursor.delete_letter();
            }
            Input::DeleteWord => {
                cursor.delete_word();
            }
        }
    }

    let position = cursor.position();

    for _ in position..expected.len() {
        kinds.push(InputKind::Missed);
    }

    (kinds, position, position == expected.len())
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
                    let (expected_text, inputs, expected_kinds, expected_cursor, expected_done) = $value;
                    assert_eq!((expected_kinds, expected_cursor, expected_done), correct_input_naive(expected_text, &inputs));
                }
            )*
            }
        }

        test_cases! {
            empty_input: (
                "hello",
                inputs![],
                correction![- - - - -],
                0,
                false
            ),

            correct_simple: (
                "hello",
                inputs!["hello"],
                correction![h e l l o],
                5,
                true
            ),

            incorrect_simple: (
                "hello",
                inputs!["hxlyo"],
                correction![h & l & o],
                5,
                true
            ),

            additional_simple: (
                "hello",
                inputs!["hellor"],
                correction![h e l l o +],
                6,
                false
            ),

            with_delete_letter: (
                "hello",
                inputs!["hx", delete_letter, "el"],
                correction![h & e l - -],
                3,
                false
            ),

            empty_input_with_delete_letter: (
                "",
                inputs![delete_letter],
                correction![],
                0,
                true
            ),

            with_delete_word: (
                "hello world",
                inputs!["hello wx", delete_word, "wo"],
                correction![h e l l o _ w & w o - - -],
                8,
                false
            ),

            empty_input_with_delete_word: (
                "",
                inputs![delete_word],
                correction![],
                0,
                true
            ),

            with_delete_word_with_double_space_before: (
                "hello world",
                inputs!["hello  w", delete_word, delete_letter, "wo"],
                correction![h e l l o _ & & w o - - -],
                8,
                false
            ),

            with_delete_word_with_double_space_after: (
                "hello",
                inputs!["hello  ", delete_word],
                correction![h e l l o + + - - - - -],
                0,
                false
            ),

            delete_word_from_beginning: (
                "test word",
                inputs!["tesx", delete_word, "test"],
                correction![t e s & t e s t - - - - -],
                4,
                false
            ),
        }
    }

    mod cursor_iterator {
        use super::*;

        macro_rules! test_cases  {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (inputs, expected_positions) = $value;
                    assert_eq!(expected_positions.len(), inputs.len() + 1, "expected positions should have one more value than inputs");

                    let mut cursor = Cursor::new();
                    let iter = expected_positions.iter().zip(&inputs);

                    for (&expected_position, input) in iter {
                        assert_eq!(cursor.position(), expected_position);

                        match input {
                            Input::Character(c) => {
                                cursor.input(*c);
                            }
                            Input::DeleteLetter => {
                                cursor.delete_letter();
                            }
                            Input::DeleteWord => {
                                cursor.delete_word();
                            }
                        }
                    }

                    if let Some(&last) = expected_positions.last() {
                        assert_eq!(cursor.position(), last);
                    }
                }
            )*
            }
        }

        test_cases! {
            empty_input: (
                inputs![],
                [0]
            ),

            single_character: (
                inputs!["h"],
                [0, 1]
            ),

            multiple_characters: (
                inputs!["hello"],
                [0, 1, 2, 3, 4, 5]
            ),

            delete_letter_basic: (
                inputs!["hi", delete_letter],
                [0, 1, 2, 1]
            ),

            delete_letter_at_beginning: (
                inputs![delete_letter, "h"],
                [0, 0, 1]
            ),

            delete_letter_with_space: (
                inputs!["hi ", delete_letter],
                [0, 1, 2, 3, 2]
            ),

            delete_word_basic: (
                inputs!["hello world", delete_word],
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 6]
            ),

            delete_word_at_beginning: (
                inputs![delete_word, "h"],
                [0, 0, 1]
            ),

            delete_word_multiple_spaces: (
                inputs!["hello  world", delete_word],
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 7]
            ),

            complex_sequence: (
                inputs!["hel", delete_letter, "llo wo", delete_word, "world"],
                [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 6, 7, 8, 9, 10, 11]
            ),
        }
    }
}
