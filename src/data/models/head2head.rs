use rusqlite::Row;
use serde::Deserialize;
use skillratings::Outcomes;

#[derive(Debug, Deserialize)]
pub struct Head2Head {
    pub team1: u32,
    pub team2: u32,
    pub total_games: u32,
    pub team_wins: u32,
    pub team_win_freq: f64,
}

impl Head2Head {
    pub fn update(&mut self, won: Outcomes) {
        if won == Outcomes::WIN {
            self.team_wins += 1;
        }
        self.total_games += 1;
        self.team_win_freq = self.team_wins as f64 / self.total_games as f64;
    }
}

impl TryFrom<&Row<'_>> for Head2Head {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Head2Head {
            team1: row.get(0)?,
            team2: row.get(1)?,
            total_games: row.get(2)?,
            team_wins: row.get(3)?,
            team_win_freq: row.get(4)?,
        })
    }
}
