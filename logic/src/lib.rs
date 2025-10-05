pub mod generator;

use std::time::{Duration, Instant};

use itertools::Itertools;

#[derive(Debug, PartialEq)]
pub enum Input {
    Character(char),
    DeleteLetter,
    DeleteWord,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputKind {
    Correct,
    Incorrect,
    Additional,
    Missed,
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

#[derive(Default)]
pub struct InputText {
    text: String,
}

impl InputText {
    pub fn character(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn delete_letter(&mut self) {
        self.text.pop();
    }

    pub fn delete_word(&mut self) {
        while let Some(c) = self.text.pop() {
            if !c.is_whitespace() {
                self.text.push(c);
                break;
            }
        }

        while let Some(c) = self.text.pop() {
            if c.is_whitespace() {
                self.text.push(c);
                break;
            }
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.text.len()
    }
}

impl From<InputText> for String {
    fn from(value: InputText) -> Self {
        value.text
    }
}

pub fn correct_input_naive(
    expected: &str,
    inputs: &[Input],
) -> (Vec<usize>, Vec<InputKind>, String, bool) {
    let mut text = InputText::default();
    let chars = expected.chars().collect_vec();
    let mut positions = Vec::<usize>::new();
    let mut kinds = Vec::<InputKind>::new();

    for input in inputs {
        match input {
            Input::Character(c) => {
                positions.push(text.cursor());
                if let Some(expected_c) = chars.get(text.cursor()) {
                    if expected_c == c {
                        kinds.push(InputKind::Correct);
                    } else {
                        kinds.push(InputKind::Incorrect);
                    }
                } else {
                    kinds.push(InputKind::Additional);
                }
                text.character(*c);
            }
            Input::DeleteLetter => {
                text.delete_letter();
            }
            Input::DeleteWord => {
                text.delete_word();
            }
        }
    }

    let position = text.cursor();

    for p in position..expected.len() {
        positions.push(p);
        kinds.push(InputKind::Missed);
    }

    (positions, kinds, text.into(), position == expected.len())
}

pub fn output_text<'a>(
    positions: &[usize],
    corrections: &[InputKind],
    typed: &'a str,
) -> Vec<(InputKind, &'a str)> {
    assert_eq!(positions.len(), corrections.len());

    let mut errors = Vec::<InputKind>::new();
    errors.resize(typed.len(), InputKind::Missed);

    for (&position, &correction) in positions.iter().zip(corrections.iter()) {
        errors[position] = correction;
    }

    let mut output = Vec::<(InputKind, &str)>::new();
    let mut group_kind = InputKind::Correct;
    let mut group_start = 0;

    for (position, kind) in errors.into_iter().enumerate() {
        if group_kind != kind {
            if group_start < position {
                output.push((group_kind, &typed[group_start..position]));
            }
            group_kind = kind;
            group_start = position;
        }
    }

    output
}

#[derive(Default)]
pub struct InputErrorCounts {
    correct: usize,
    incorrect: usize,
    additional: usize,
    missed: usize,
}

impl InputErrorCounts {
    pub fn accuracy(&self) -> f64 {
        self.correct as f64 / (self.correct + self.incorrect + self.additional + self.missed) as f64
    }
}

impl From<&[InputKind]> for InputErrorCounts {
    fn from(value: &[InputKind]) -> Self {
        value
            .iter()
            .fold(InputErrorCounts::default(), |mut counts, kind| {
                match kind {
                    InputKind::Correct => {
                        counts.correct += 1;
                    }
                    InputKind::Incorrect => {
                        counts.incorrect += 1;
                    }
                    InputKind::Additional => {
                        counts.additional += 1;
                    }
                    InputKind::Missed => {
                        counts.missed += 1;
                    }
                }
                counts
            })
    }
}

pub fn test_duration(timestamps: &[Instant]) -> Duration {
    if let (Some(&start), Some(&end)) = (timestamps.first(), timestamps.last()) {
        end - start
    } else {
        Duration::new(0, 0)
    }
}

pub fn cpm(number_of_inputs: usize, test_duration: Duration) -> f64 {
    number_of_inputs as f64 / (test_duration.as_secs_f64() / 60.0)
}

pub fn cpm_from_timestamps(timestamps: &[Instant]) -> f64 {
    cpm(timestamps.len(), test_duration(timestamps))
}

pub fn wpm_from_cpm(cpm: f64) -> f64 {
    cpm / 5.0
}

pub fn wpm(number_of_inputs: usize, test_duration: Duration) -> f64 {
    wpm_from_cpm(cpm(number_of_inputs, test_duration))
}

pub fn wpm_from_timestamps(timestamps: &[Instant]) -> f64 {
    wpm_from_cpm(cpm_from_timestamps(timestamps))
}

fn keypresses_each_second(timestamps: &[Instant]) -> Vec<usize> {
    let mut iter = timestamps.iter();
    if let Some(first) = iter.next() {
        let mut history = Vec::<usize>::new();
        let mut start = *first;
        let mut count = 1;

        for &time in iter {
            if (time - start).as_secs_f64() >= 1.0 {
                history.push(count);
                count = 1;
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

pub fn consistency_by_second(timestamps: &[Instant]) -> f64 {
    let history = keypresses_each_second(timestamps);
    let mean = (history.iter().sum::<usize>() as f64) / (history.len() as f64);
    let standard_deviation = history
        .iter()
        .map(|&count| ((count as f64) - mean).powi(2))
        .sum::<f64>()
        / (history.len() as f64);

    standard_deviation.sqrt() / mean
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

    macro_rules! timestamps {
        () => {{
            let start = Instant::now();
            vec![start]
        }};

        (@build_durations $durations:ident) => {};

        (@build_durations $durations:ident, $count:literal * $duration:literal $(, $($rest:tt)*)?) => {
            for _ in 0..$count {
                $durations.push($duration);
            }
            timestamps!(@build_durations $durations $(, $($rest)*)?);
        };

        (@build_durations $durations:ident, $duration:literal $(, $($rest:tt)*)?) => {
            $durations.push($duration);
            timestamps!(@build_durations $durations $(, $($rest)*)?);
        };

        ($($input:tt)*) => {{
            let start = Instant::now();
            let mut durations = Vec::new();
            timestamps!(@build_durations durations, $($input)*);
            let mut result = vec![start];
            let mut current = start;
            for duration in durations {
                current += Duration::from_millis(duration);
                result.push(current);
            }
            result
        }};
    }

    #[test]
    fn test_timestamps_macro() {
        let ts1 = timestamps![100, 200, 300];
        assert_eq!(ts1.len(), 4); // start + 3 durations

        let ts2 = timestamps![4 * 200];
        assert_eq!(ts2.len(), 5); // start + 4 durations

        let ts3 = timestamps![2 * 100, 500, 3 * 50];
        assert_eq!(ts3.len(), 7); // start + 2 + 1 + 3 durations
    }

    mod correct_input_naive {
        use super::*;

        macro_rules! test_cases  {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected_text, inputs, expected_kinds, expected_output, expected_done): (&str, Vec<Input>, Vec<InputKind>, &str, bool) = $value;
                    let (_, kinds, output, is_done) = correct_input_naive(expected_text, &inputs);
                    assert_eq!(expected_kinds, kinds);
                    assert_eq!(expected_output, &output);
                    assert_eq!(expected_done, is_done);
                }
            )*
            }
        }

        test_cases! {
            empty_input: (
                "hello",
                inputs![],
                correction![- - - - -],
                "",
                false
            ),

            correct_simple: (
                "hello",
                inputs!["hello"],
                correction![h e l l o],
                "hello",
                true
            ),

            incorrect_simple: (
                "hello",
                inputs!["hxlyo"],
                correction![h & l & o],
                "hxlyo",
                true
            ),

            additional_simple: (
                "hello",
                inputs!["hellor"],
                correction![h e l l o +],
                "hellor",
                false
            ),

            with_delete_letter: (
                "hello",
                inputs!["hx", delete_letter, "el"],
                correction![h & e l - -],
                "hel",
                false
            ),

            empty_input_with_delete_letter: (
                "",
                inputs![delete_letter],
                correction![],
                "",
                true
            ),

            with_delete_word: (
                "hello world",
                inputs!["hello wx", delete_word, "wo"],
                correction![h e l l o _ w & w o - - -],
                "hello wo",
                false
            ),

            empty_input_with_delete_word: (
                "",
                inputs![delete_word],
                correction![],
                "",
                true
            ),

            with_delete_word_with_double_space_before: (
                "hello world",
                inputs!["hello  w", delete_word, delete_letter, "wo"],
                correction![h e l l o _ & & w o - - -],
                "hello wo",
                false
            ),

            with_delete_word_with_double_space_after: (
                "hello",
                inputs!["hello  ", delete_word],
                correction![h e l l o + + - - - - -],
                "",
                false
            ),

            delete_word_from_beginning: (
                "test word",
                inputs!["tesx", delete_word, "test"],
                correction![t e s & t e s t - - - - -],
                "test",
                false
            ),
        }
    }

    mod cursor {
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

    mod input_error_counts {
        use super::*;

        macro_rules! test_cases {
            ($($name:ident: ($input:expr, correct: $correct:expr, incorrect: $incorrect:expr, additional: $additional:expr, missed: $missed:expr),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let input = $input;
                        let counts = InputErrorCounts::from(input.as_slice());
                        assert_eq!(counts.correct, $correct);
                        assert_eq!(counts.incorrect, $incorrect);
                        assert_eq!(counts.additional, $additional);
                        assert_eq!(counts.missed, $missed);
                    }
                )*
            };
        }

        test_cases! {
            empty_input: (
                correction![],
                correct: 0,
                incorrect: 0,
                additional: 0,
                missed: 0
            ),

            single_correct: (
                correction![c],
                correct: 1,
                incorrect: 0,
                additional: 0,
                missed: 0
            ),

            single_incorrect: (
                correction![&],
                correct: 0,
                incorrect: 1,
                additional: 0,
                missed: 0
            ),

            single_additional: (
                correction![+],
                correct: 0,
                incorrect: 0,
                additional: 1,
                missed: 0
            ),

            single_missed: (
                correction![-],
                correct: 0,
                incorrect: 0,
                additional: 0,
                missed: 1
            ),

            mixed_input: (
                correction![c c & + - - c],
                correct: 3,
                incorrect: 1,
                additional: 1,
                missed: 2
            ),
        }

        #[test]
        fn default_creates_zero_counts() {
            let counts = InputErrorCounts::default();
            assert_eq!(counts.correct, 0);
            assert_eq!(counts.incorrect, 0);
            assert_eq!(counts.additional, 0);
            assert_eq!(counts.missed, 0);
        }
    }

    mod test_duration {
        use super::*;
        use std::time::{Duration, Instant};

        macro_rules! test_cases {
            ($($name:ident: (
                $timestamps:expr,
                $expected_ms:expr
            ),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let timestamps = $timestamps;
                        let result = test_duration(&timestamps);
                        assert_eq!(result, Duration::from_millis($expected_ms));
                    }
                )*
            };
        }

        test_cases! {
            empty_timestamps: (
                vec![],
                0
            ),

            single_timestamp: (
                timestamps![],
                0
            ),

            two_timestamps_5_seconds: (
                timestamps![5000],
                5000
            ),

            three_timestamps_7_seconds: (
                timestamps![2000, 5000],
                7000
            ),

            multiple_intervals: (
                timestamps![1000, 500, 2000, 1500],
                5000
            ),
        }
    }

    mod cpm {
        use super::*;
        use std::time::{Duration, Instant};

        macro_rules! test_cases {
            ($($name:ident: (inputs: $inputs:expr, duration_secs: $duration_secs:expr, expected: $expected:expr),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let duration = Duration::from_secs($duration_secs);
                        let result = cpm($inputs, duration);
                        assert_eq!(result, $expected);
                    }
                )*
            };
        }

        test_cases! {
            basic_calculation: (inputs: 120, duration_secs: 60, expected: 120.0),
            half_minute: (inputs: 60, duration_secs: 30, expected: 120.0),
            two_minutes: (inputs: 120, duration_secs: 120, expected: 60.0),
            zero_inputs: (inputs: 0, duration_secs: 60, expected: 0.0),
        }

        #[test]
        fn from_timestamps_basic() {
            let timestamps = timestamps![60000];
            assert_eq!(cpm_from_timestamps(&timestamps), 2.0);
        }

        #[test]
        fn from_timestamps_empty() {
            let timestamps: Vec<Instant> = vec![];
            assert!(cpm_from_timestamps(&timestamps).is_nan());
        }

        #[test]
        fn from_timestamps_single() {
            let timestamps = timestamps![];
            assert!(cpm_from_timestamps(&timestamps).is_infinite());
        }
    }

    mod wpm {
        use super::*;
        use std::time::{Duration, Instant};

        #[test]
        fn from_cpm() {
            assert_eq!(wpm_from_cpm(100.0), 20.0);
            assert_eq!(wpm_from_cpm(60.0), 12.0);
            assert_eq!(wpm_from_cpm(300.0), 60.0);
            assert_eq!(wpm_from_cpm(0.0), 0.0);
        }

        macro_rules! test_cases {
            ($($name:ident: (inputs: $inputs:expr, duration_secs: $duration_secs:expr, expected: $expected:expr),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let duration = Duration::from_secs($duration_secs);
                        let result = wpm($inputs, duration);
                        assert_eq!(result, $expected);
                    }
                )*
            };
        }

        test_cases! {
            basic_calculation: (inputs: 300, duration_secs: 60, expected: 60.0),
            half_minute: (inputs: 150, duration_secs: 30, expected: 60.0),
            two_minutes: (inputs: 600, duration_secs: 120, expected: 60.0),
            zero_inputs: (inputs: 0, duration_secs: 60, expected: 0.0),
        }

        #[test]
        fn from_timestamps() {
            let timestamps = timestamps![100, 100, 100, 100, 59600];
            let expected_wpm = (6.0 / 60.0) / 5.0 * 60.0;
            assert_eq!(wpm_from_timestamps(&timestamps), expected_wpm);
        }

        #[test]
        fn from_timestamps_empty() {
            let timestamps: Vec<Instant> = vec![];
            assert!(wpm_from_timestamps(&timestamps).is_nan());
        }

        #[test]
        fn from_timestamps_single() {
            let timestamps = timestamps![];
            assert!(wpm_from_timestamps(&timestamps).is_infinite());
        }
    }

    mod consistency_by_second {
        use super::*;
        use std::time::{Duration, Instant};

        macro_rules! test_cases {
            ($($name:ident: (
                $timestamps:expr,
                $expected:expr
            ),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let timestamps: Vec<Instant> = $timestamps;
                        let result = consistency_by_second(&timestamps);
                        println!("keypresses: {:?}", keypresses_each_second(&timestamps));
                        assert!((result - $expected).abs() < 0.01, "result is not as expected\n expected: {}\n   result: {}", $expected, result);
                    }
                )*
            };
        }

        #[test]
        fn empty_timestamps() {
            let timestamps: Vec<Instant> = vec![];
            assert!(consistency_by_second(&timestamps).is_nan());
        }

        test_cases! {
            single_timestamp: (timestamps![], 0.0),
            perfect_consistency: (timestamps![19 * 100], 0.0),
            varying_speed: (timestamps![4 * 400, 800, 6 * 100], 0.54),
            two_seconds_equal_speed: (timestamps![3 * 100], 0.0),
        }
    }

    mod keypresses_each_second {
        use super::*;
        use std::time::{Duration, Instant};

        macro_rules! test_cases {
            ($($name:ident: (
                $timestamps:expr,
                $expected:expr
            ),)*) => {
                $(
                    #[test]
                    fn $name() {
                        let timestamps = $timestamps;
                        let result = keypresses_each_second(&timestamps);
                        assert_eq!(result, $expected);
                    }
                )*
            };
        }

        test_cases! {
            empty_timestamps: (
                vec![],
                vec![]
            ),
            single_timestamp: (
                timestamps![],
                vec![1]
            ),
            two_timestamps_same_second: (
                timestamps![500],
                vec![2]
            ),
            two_timestamps_different_seconds: (
                timestamps![1500],
                vec![1, 1]
            ),
            multiple_seconds_with_varying_counts: (
                timestamps![200, 200, 700, 200, 200, 200, 400],
                vec![3, 4, 1]
            ),
            exactly_one_second_apart: (
                timestamps![1000, 1000],
                vec![1, 1, 1]
            ),
        }
    }
}
