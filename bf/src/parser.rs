use std::str::Chars;

pub struct Parser<'c> {
    chars: Chars<'c>,
    current: char,
}

impl<'c> Parser<'c> {
    pub fn new(contents: &'c str) -> Self {
        let mut chars = contents.chars();
        let current = chars.next().unwrap_or('\0');

        Self { chars, current }
    }

    fn advance(&mut self) {
        self.current = self.chars.next().unwrap_or('\0');
    }
}

pub enum Instruction {
    MoveLeft,
    MoveRight,
    StartLoop,
    CloseLoop,
    Advance,
    Decrease,
    Output,
    Input,
    EndOfInput,
}

impl<'c> Iterator for Parser<'c> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current {
                '\0' => return Some(Instruction::EndOfInput),
                '+' => return Some(Instruction::Advance),
                '-' => return Some(Instruction::Decrease),
                '>' => return Some(Instruction::MoveRight),
                '<' => return Some(Instruction::MoveLeft),
                '.' => return Some(Instruction::Output),
                ',' => return Some(Instruction::Input),
                '[' => return Some(Instruction::StartLoop),
                ']' => return Some(Instruction::CloseLoop),
                _ => {}
            }

            self.advance();
        }
    }
}
