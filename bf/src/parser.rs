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

#[derive(PartialEq, Debug)]
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

impl Default for Instruction {
    fn default() -> Self {
        Self::EndOfInput
    }
}

impl<'c> Iterator for Parser<'c> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let instruction = match self.current {
                '\0' => Some(Instruction::EndOfInput),
                '+' => Some(Instruction::Advance),
                '-' => Some(Instruction::Decrease),
                '>' => Some(Instruction::MoveRight),
                '<' => Some(Instruction::MoveLeft),
                '.' => Some(Instruction::Output),
                ',' => Some(Instruction::Input),
                '[' => Some(Instruction::StartLoop),
                ']' => Some(Instruction::CloseLoop),
                _ => None,
            };

            self.advance();

            if instruction.is_some() {
                return instruction;
            }
        }
    }
}
