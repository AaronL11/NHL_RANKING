use rusqlite::Row;
use serde::{Deserialize, Deserializer};
use skillratings::weng_lin::WengLinRating;

use crate::models::players::{PlayerStats, collect_players};

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
