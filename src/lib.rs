//! Pravila igre su odvojena od prikaza, kako bi se lako proveravala i proširivala.

pub const LEVEL_LIMITS: [u8; 3] = [10, 15, 20];
pub const QUESTIONS_PER_LEVEL: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayStyle {
    Levels,
    PracticeTo10,
}

impl PlayStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Levels => "Са нивоима",
            Self::PracticeTo10 => "Вежба до 10",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Addition,
    Subtraction,
    Mixed,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Addition => "Сабирање",
            Self::Subtraction => "Одузимање",
            Self::Mixed => "Оба",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
}

#[derive(Clone, Debug)]
pub struct Question {
    pub left: u8,
    pub right: u8,
    pub operation: Operation,
    pub choices: [u8; 3],
}

impl Question {
    pub fn answer(&self) -> u8 {
        match self.operation {
            Operation::Add => self.left + self.right,
            Operation::Subtract => self.left - self.right,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self.operation {
            Operation::Add => "+",
            Operation::Subtract => "−",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Correct,
    Finished,
}

pub struct Game {
    pub mode: Mode,
    pub style: PlayStyle,
    pub level: usize,
    pub completed: u64,
    pub score: u64,
    pub phase: Phase,
    pub question: Question,
    pub tried: Vec<u8>,
    pub feedback: String,
    random: Random,
}

impl Game {
    pub fn new(mode: Mode, seed: u64) -> Self {
        Self::with_style(mode, PlayStyle::Levels, seed)
    }

    pub fn with_style(mode: Mode, style: PlayStyle, seed: u64) -> Self {
        let mut game = Self {
            mode,
            style,
            level: 0,
            completed: 0,
            score: 0,
            phase: Phase::Playing,
            question: Question {
                left: 0,
                right: 0,
                operation: Operation::Add,
                choices: [0, 1, 2],
            },
            tried: Vec::new(),
            feedback: String::new(),
            random: Random(seed.max(1)),
        };
        game.generate();
        game
    }

    pub fn submit(&mut self, answer: u8) {
        if self.phase != Phase::Playing
            || !self.question.choices.contains(&answer)
            || self.tried.contains(&answer)
        {
            return;
        }
        if answer == self.question.answer() {
            let points = if self.tried.is_empty() { 10 } else { 5 };
            self.score = self.score.saturating_add(points);
            self.completed = self.completed.saturating_add(1);
            self.phase = Phase::Correct;
            self.feedback = format!("Браво! +{points} бодова");
        } else {
            self.tried.push(answer);
            self.feedback = "Хајде да пребројимо поново!".into();
        }
    }

    pub fn advance(&mut self) {
        if self.phase != Phase::Correct {
            return;
        }
        if self.style == PlayStyle::Levels && self.completed == u64::from(QUESTIONS_PER_LEVEL) {
            if self.level + 1 == LEVEL_LIMITS.len() {
                self.phase = Phase::Finished;
                return;
            }
            self.level += 1;
            self.completed = 0;
        }
        self.phase = Phase::Playing;
        self.generate();
    }

    pub fn limit(&self) -> u8 {
        match self.style {
            PlayStyle::Levels => LEVEL_LIMITS[self.level],
            PlayStyle::PracticeTo10 => 10,
        }
    }

    fn generate(&mut self) {
        let limit = self.limit();
        let operation = match self.mode {
            Mode::Addition => Operation::Add,
            Mode::Subtraction => Operation::Subtract,
            Mode::Mixed => {
                if self.random.below(2) == 0 {
                    Operation::Add
                } else {
                    Operation::Subtract
                }
            }
        };
        // The left group is always nonempty. Addition introduces zero only on
        // later levels; subtraction includes zero as a possible result.
        let left = 1 + self.random.below(if operation == Operation::Add {
            limit - 1
        } else {
            limit
        });
        let right = match operation {
            Operation::Add => {
                let minimum = u8::from(self.level == 0);
                minimum + self.random.below(limit - left - minimum + 1)
            }
            Operation::Subtract => 1 + self.random.below(left),
        };
        let answer = if operation == Operation::Add {
            left + right
        } else {
            left - right
        };
        let mut choices = vec![answer];
        while choices.len() < 3 {
            let candidate = self.random.below(limit + 1);
            if !choices.contains(&candidate) {
                choices.push(candidate);
            }
        }
        for i in (1..3).rev() {
            let j = self.random.below((i + 1) as u8) as usize;
            choices.swap(i, j);
        }
        self.question = Question {
            left,
            right,
            operation,
            choices: [choices[0], choices[1], choices[2]],
        };
        self.tried.clear();
        self.feedback = "Изабери одговор".into();
    }
}

// A seeded generator makes arithmetic tests reproducible; no security use.
struct Random(u64);

impl Random {
    fn below(&mut self, upper: u8) -> u8 {
        assert!(upper > 0);
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % u64::from(upper)) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_stays_within_ten_without_levels_or_an_end() {
        for mode in [Mode::Addition, Mode::Subtraction, Mode::Mixed] {
            let mut game = Game::with_style(mode, PlayStyle::PracticeTo10, 42);
            for completed in 0..7_000 {
                assert_eq!(game.phase, Phase::Playing);
                assert_eq!(game.level, 0);
                assert_eq!(game.completed, completed);
                assert_eq!(game.limit(), 10);
                let question = &game.question;
                assert!(question.left <= 10 && question.right <= 10);
                assert!(question.answer() <= 10);
                assert!(question.choices.iter().all(|&choice| choice <= 10));
                match mode {
                    Mode::Addition => assert_eq!(question.operation, Operation::Add),
                    Mode::Subtraction => assert_eq!(question.operation, Operation::Subtract),
                    Mode::Mixed => {}
                }
                game.submit(question.answer());
                game.advance();
            }
            assert_eq!(game.score, 70_000);
        }
    }

    #[test]
    fn all_questions_have_safe_operands_and_unique_bounded_choices() {
        for mode in [Mode::Addition, Mode::Subtraction, Mode::Mixed] {
            for seed in 0..200 {
                let mut game = Game::new(mode, seed);
                while game.phase != Phase::Finished {
                    let q = &game.question;
                    let limit = LEVEL_LIMITS[game.level];
                    assert!(q.left <= limit && q.right <= limit);
                    if q.operation == Operation::Subtract {
                        assert!(q.left >= q.right);
                    }
                    if mode == Mode::Addition {
                        assert_eq!(q.operation, Operation::Add);
                    }
                    if mode == Mode::Subtraction {
                        assert_eq!(q.operation, Operation::Subtract);
                    }
                    assert!(q.answer() <= limit);
                    assert!(q.choices.contains(&q.answer()));
                    assert!(q.choices.iter().all(|&value| value <= limit));
                    assert!(
                        q.choices[0] != q.choices[1]
                            && q.choices[1] != q.choices[2]
                            && q.choices[0] != q.choices[2]
                    );
                    game.submit(q.answer());
                    game.advance();
                }
                assert_eq!(game.score, 150);
            }
        }
    }

    #[test]
    fn retry_is_encouraged_and_a_question_cannot_score_twice() {
        let mut game = Game::new(Mode::Addition, 42);
        let correct = game.question.answer();
        let wrong = *game
            .question
            .choices
            .iter()
            .find(|&&n| n != correct)
            .unwrap();
        game.advance();
        assert_eq!(game.completed, 0);
        game.submit(wrong);
        game.submit(wrong);
        assert_eq!(game.tried.len(), 1);
        assert_eq!(game.score, 0);
        game.submit(255);
        assert_eq!(game.tried.len(), 1);
        game.submit(correct);
        game.submit(correct);
        assert_eq!(game.score, 5);
        assert_eq!(game.completed, 1);
        game.advance();
        assert!(game.tried.is_empty());
        assert_eq!(game.phase, Phase::Playing);
    }

    #[test]
    fn levels_advance_only_after_five_completed_questions() {
        let mut game = Game::new(Mode::Mixed, 100);
        for level in 0..3 {
            for completed in 0..5 {
                assert_eq!(game.level, level);
                assert_eq!(game.completed, completed);
                game.submit(game.question.answer());
                assert_eq!(game.phase, Phase::Correct);
                game.advance();
            }
        }
        assert_eq!(game.phase, Phase::Finished);
        game.submit(game.question.answer());
        game.advance();
        assert_eq!(game.score, 150);
    }
}
