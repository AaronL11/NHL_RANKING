use rusqlite::Row;
use serde::Deserialize;
use skillratings::weng_lin::WengLinRating;

use crate::data::models::head2head::Head2Head;

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: u32,

    pub name: String,

    pub abbrev: String,

    pub rating: WengLinRating,
}

impl Team {
    pub fn update(&mut self, rating: WengLinRating) {
        self.rating = rating;
    }

    // pub fn from_row(row)
    pub fn vs(&self, team: &Team) -> Head2Head {
        Head2Head {
            team1: self.id,
            team2: team.id,
            total_games: 0,
            team_wins: 0,
            team_win_freq: 0.,
        }
    }
}

impl TryFrom<&Row<'_>> for Team {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Team {
            id: row.get(0)?,
            name: row.get(1)?,
            abbrev: row.get(2)?,
            rating: WengLinRating {
                rating: row.get(3)?,
                uncertainty: row.get(4)?,
            },
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct TeamsResponse {
    #[serde(rename = "data")]
    pub teams: Vec<TeamResponse>,
}

#[derive(Debug, Deserialize)]
pub struct TeamResponse {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "franchiseId")]
    pub franchise_id: Option<i64>,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "leagueId")]
    pub league_id: u64,
    #[serde(rename = "rawTricode")]
    pub raw_tricode: String,
    #[serde(rename = "triCode")]
    pub tri_code: String,
}
