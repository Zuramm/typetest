use std::time::Instant;

pub enum Input {
    Character(char),
    DeleteLetter,
    DeleteWord,
}

pub enum InputKind {
    Correct,
    Incorrect,
    Additional,
    Untyped,
}

pub struct InputResult {
    kind: InputKind,
    is_incorrect: bool,
    is_commiting: bool,
    is_done: bool,
}

pub struct RunningTypingTest {
    expected: Vec<char>,
    inputs: Vec<(Instant, Input)>,
}

impl RunningTypingTest {
    fn new(expected: String) -> Self {
        Self {
            expected: expected.chars().collect(),
            inputs: vec![],
        }
    }

    fn cursor_pos(&self) -> usize {
        todo!();
    }

    fn input(&mut self, time: Instant, input: Input) -> InputResult {
        todo!();
    }
}

