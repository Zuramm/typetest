pub mod generator;

use std::time::{Duration, Instant};

use itertools::Itertools;

pub enum Input {
    Character(char),
    DeleteLetter,
    DeleteWord,
}

pub enum InputKind {
    Correct,
    Incorrect,
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
