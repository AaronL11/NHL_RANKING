use nhl_api::{Boxscore, GameScore, ScheduleGame};

use crate::data::{db::TeamID, models::teams::Team};

pub struct Game {
    pub id: i64,
    pub away_id: TeamID,
    pub home_id: TeamID,
    pub score: (u32, u32),
}

impl Game {
    pub fn ids(&self) -> (TeamID, TeamID) {
        (self.away_id, self.home_id)
    }
}

impl From<&ScheduleGame> for Game {
    fn from(game: &ScheduleGame) -> Self {
        let (away_team, home_team) = (&game.away_team, &game.home_team);
        let score = (
            away_team.score.unwrap() as u32,
            home_team.score.unwrap() as u32,
        );
        Game {
            id: game.id,
            away_id: away_team.id,
            home_id: home_team.id,
            score,
        }
    }
}

impl From<&GameScore> for Game {
    fn from(game: &GameScore) -> Self {
        let (away_team, home_team) = (&game.away_team, &game.home_team);
        let score = (
            away_team.score.unwrap() as u32,
            home_team.score.unwrap() as u32,
        );
        Game {
            id: game.id,
            away_id: away_team.id,
            home_id: home_team.id,
            score,
        }
    }
}

impl From<&Boxscore> for Game {
    fn from(game: &Boxscore) -> Self {
        let (away_team, home_team) = (&game.away_team, &game.home_team);
        let score = (away_team.score as u32, home_team.score as u32);
        Game {
            id: game.id,
            away_id: away_team.id,
            home_id: home_team.id,
            score,
        }
    }
}
