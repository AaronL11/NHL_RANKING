use std::collections::VecDeque;

use log::debug;
use rusqlite::Row;
use skillratings::Outcomes;

#[derive(Debug)]
pub struct Last10 {
    pub id: u32,
    pub wins: u32,
    pub loss: u32,
    pub games: VecDeque<Outcomes>,
}

impl Last10 {
    pub fn update(&mut self, outcome: Outcomes) {
        if outcome == Outcomes::WIN {
            self.wins += 1;
        } else {
            self.loss += 1;
        }
        self.games.push_back(outcome);
        if let Some(outcome_old) = self.games.pop_front() {
            if outcome_old == Outcomes::WIN {
                self.wins -= 1;
            } else if self.loss > 0 {
                self.loss -= 1;
            }
        }
    }

    pub fn estimate(&self) -> f64 {
        self.wins as f64 / 10.
    }
}

impl TryFrom<&Row<'_>> for Last10 {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let mut games = VecDeque::new();
        let int_games: u32 = row.get(3)?;
        let mut i = 10;
        while i >= 0 {
            let game = (int_games >> i) & 1;
            let outcome = if game == 1 {
                Outcomes::WIN
            } else {
                Outcomes::LOSS
            };
            games.push_back(outcome);
            i -= 1;
        }
        Ok(Last10 {
            id: row.get(0)?,
            wins: row.get(1)?,
            loss: row.get(2)?,
            games,
        })
    }
}
