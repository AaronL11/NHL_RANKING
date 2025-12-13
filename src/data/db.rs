use crate::models::teams::Team;
use log::debug;
use rusqlite::{Connection, Result, params};
use skillratings::weng_lin::WengLinRating;

type TeamID = i64;

#[derive(Debug)]
pub struct DataBase(Connection);

impl DataBase {
    pub fn new(path: &str) -> Result<DataBase> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
        CREATE TABLE IF NOT EXISTS teams (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            abbreviation TEXT NOT NULL,
            rating REAL DEFAULT 25.0,
            uncertainty REAL DEFAULT 8.33
        );
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY,
            date TEXT NOT NULL,
            home_team_id INTEGER NOT NULL,
            away_team_id INTEGER NOT NULL,
            FOREIGN KEY (home_team_id) REFERENCES teams(id),
            FOREIGN KEY (away_team_id) REFERENCES teams(id)
        );
        ",
        )?;
        Ok(DataBase(conn))
    }

    pub fn clear(&self) -> Result<()> {
        self.0.execute_batch(
            "
            DROP TABLE IF EXISTS teams;
            DROP TABLE IF EXISTS games;
            DROP TABLE IF EXISTS players;
        ",
        )
    }

    pub fn add_team(&self, id: impl Into<TeamID>, name: String, abbrev: String) -> Result<()> {
        let id = id.into();
        debug!("Adding {} to database with id {}", name, id);
        let conn = &self.0;
        conn.execute(
            "INSERT OR IGNORE INTO teams (id, name, abbreviation) VALUES (?1, ?2, ?3)",
            params![id, name, abbrev],
        )?;
        Ok(())
    }

    pub fn get_team(&self, id: impl Into<TeamID>) -> Result<Team> {
        let id = id.into();
        debug!("Retrieving team with id: {} from the database ..", id);
        let conn = &self.0;
        let team = conn.query_row("SELECT * FROM teams WHERE id = ?1", params![id], |row| {
            debug!("Retrieving row: {:?}", row);
            Team::try_from(row)
            // Ok(Team {
            //     id: row.get(0)?,
            //     name: row.get(1)?,
            //     abbrev: row.get(2)?,
            //     rating: WengLinRating {
            //         rating: row.get(3)?,
            //         uncertainty: row.get(4)?,
            //     },
            // })
        })?;
        debug!("Team {} found", team.name);
        Ok(team)
    }

    pub fn get_team_abbrev(&self, abbrev: &str) -> Result<Team> {
        let conn = &self.0;
        let team = conn.query_row(
            "SELECT * FROM teams WHERE abbreviation = ?1",
            params![abbrev],
            |row| {
                Ok(Team {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    abbrev: row.get(2)?,
                    rating: WengLinRating {
                        rating: row.get(3)?,
                        uncertainty: row.get(4)?,
                    },
                })
            },
        )?;
        Ok(team)
    }

    pub fn update_team_rating(
        &self,
        id: impl Into<TeamID>,
        new_rating: WengLinRating,
    ) -> Result<()> {
        let id = id.into();
        let conn = &self.0;
        conn.execute(
            "UPDATE teams SET rating = ?1, uncertainty = ?2 WHERE id = ?3",
            params![new_rating.rating, new_rating.uncertainty, id],
        )?;
        Ok(())
    }

    pub fn get_top(&self, n: u64) -> Result<Vec<Team>> {
        let conn = &self.0;
        let mut teams = Vec::with_capacity(32);
        let mut stmnt = conn.prepare("SELECT * FROM teams ORDER BY rating DESC LIMIT ?1")?;
        let mut rows = stmnt.query(params![n])?;
        while let Some(row) = rows.next()? {
            let team = Team::try_from(row)?;
            teams.push(team);
        }
        Ok(teams)
    }

    pub fn get_bot(&self, n: u64) -> Result<Vec<Team>> {
        let conn = &self.0;
        let mut teams = Vec::with_capacity(32);
        let mut stmnt = conn.prepare("SELECT * FROM teams ORDER BY rating ASC LIMIT ?1")?;
        let mut rows = stmnt.query(params![n])?;
        while let Some(row) = rows.next()? {
            let team = Team::try_from(row)?;
            teams.push(team);
        }
        Ok(teams)
    }
}

// pub fn get_or_create_team(conn: &Connection, team: &Team) -> Result<i64> {
//     conn.execute(
//         "INSERT OR IGNORE INTO teams (id, name, abbreviation) VALUES (?1, ?2, ?3)",
//         params![team.id, team.name, team.abbrev],
//     )?;

//     let id: i64 = conn.query_row(
//         "SELECT id FROM teams WHERE id = ?1",
//         params![team.id],
//         |row| row.get(0),
//     )?;
//     Ok(id)
// }

// pub fn insert_game_if_not_exists(
//     conn: &Connection,
//     game: &crate::api::models::Game,
//     home_team_id: i64,
//     away_team_id: i64,
// ) -> Result<()> {
//     conn.execute(
//         "INSERT OR IGNORE INTO games (id, date, home_team_id, away_team_id)
//          VALUES (?1, ?2, ?3, ?4)",
//         params![game.id, game.gameDate, home_team_id, away_team_id],
//     )?;
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use env_logger::Env;
    use log::info;
    use nhl_api::{Client, GameDate};
    use skillratings::{
        Outcomes,
        weng_lin::{WengLinConfig, weng_lin},
    };

    use crate::rating::openskill::SkillRating;

    use super::*;
    const PATH: &'static str = "src/data/nhl.db";

    #[tokio::test]

    async fn main() -> anyhow::Result<()> {
        // Start up the app
        // Start up the logger
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        // env_logger::init();
        // Start up the client
        let client = Client::new()?;

        //
        info!("Starting NHL Ranker...");

        info!("Fetching database");
        let db = DataBase::new(PATH)?;
        info!("Database started succesfully");

        info!("Adding Teams");
        let franchises = client.franchises().await?;
        let teams = client.teams(None).await?;
        for franchise in &franchises {
            for team in &teams {
                if team.name == franchise.full_name {
                    db.add_team(franchise.id, franchise.full_name.clone(), team.abbr.clone())?;
                }
            }
        }
        info!("Teams added succesfully");

        info!("Grabbing the montreal canadiens:");
        let team = db.get_team(1)?;
        info!(
            "Found {}, with id: {}, and abbreviation: {}",
            team.name, team.id, team.abbrev
        );

        info!("Retrieving daily scores");
        let date = NaiveDate::from_ymd_opt(2025, 12, 11).unwrap();
        let scores = client.daily_scores(Some(GameDate::Date(date))).await?;
        let game = &scores.games[0];
        let boxscore = client.boxscore(game.id).await?;
        info!("Found one game with id: {}", game.id);
        let away = &boxscore.away_team;
        let home = &boxscore.home_team;
        let outcome = if away.score > home.score {
            Outcomes::WIN
        } else {
            Outcomes::LOSS
        };
        info!(
            "This game was played between the {} and the {}:\n{} {} - {} {}",
            boxscore.away_team.common_name.default,
            boxscore.home_team.common_name.default,
            away.abbrev,
            away.score,
            home.abbrev,
            home.score,
        );
        let away_team = db.get_team_abbrev(&away.abbrev)?;
        let home_team = db.get_team_abbrev(&home.abbrev)?;
        info!(
            "The {} are rated: {}",
            away_team.name,
            // away_team.rating.rating * 100.
            away_team.rating.mmr()
        );
        info!(
            "The {} are rated: {}",
            home_team.name,
            // home_team.rating.rating * 100.
            home_team.rating.mmr()
        );
        let (new_away, new_home) = weng_lin(
            &away_team.rating,
            &home_team.rating,
            &outcome,
            &WengLinConfig::new(),
        );
        db.update_team_rating(away_team.id, new_away)?;
        db.update_team_rating(home_team.id, new_home)?;
        let away_team = db.get_team(away_team.id)?;
        let home_team = db.get_team(home_team.id)?;
        info!(
            "The {} are now rated: {}",
            away_team.name,
            away_team.rating.mmr()
        );
        info!(
            "The {} are now rated: {}",
            home_team.name,
            home_team.rating.mmr()
        );

        info!("Deleting for reuse");
        db.clear()?;
        info!("Succesfully deleted tables");
        Ok(())
    }
}
