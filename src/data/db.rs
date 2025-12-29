use crate::data::models::{head2head::Head2Head, last10::Last10, teams::Team};
use rusqlite::{Connection, Result, params};
use skillratings::{Outcomes, weng_lin::WengLinRating};

pub type TeamID = i64;

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
        CREATE TABLE IF NOT EXISTS H2H (
            awayID INTEGER,
            homeID INTEGER,
            totalGames INTEGER DEFAULT 0,
            teamWins INTEGER DEFAULT 0,
            teamWinFreq REAL DEFAULT 0.0,
            PRIMARY KEY (awayID, homeID)
        );
        CREATE TABLE IF NOT EXISTS last10 (
            id INTEGER PRIMARY KEY,
            wins INTEGER DEFAULT 0,
            losses INTEGER DEFAULT 0,
            games INTEGER DEFAULT 0
        );
        ",
        )?;
        Ok(DataBase(conn))
    }

    pub fn clear(&self) -> Result<()> {
        self.0.execute_batch(
            "
            DROP TABLE IF EXISTS teams;
            DROP TABLE IF EXISTS H2H;
            DROP TABLE IF EXISTS last10;
        ",
        )
    }

    pub fn add_team(&self, id: impl Into<TeamID>, name: String, abbrev: String) -> Result<()> {
        let id = id.into();
        let conn = &self.0;
        conn.execute(
            "INSERT OR IGNORE INTO teams (id, name, abbreviation) VALUES (?1, ?2, ?3);",
            params![id, name, abbrev],
        )?;
        Ok(())
    }

    pub fn get_team(&self, id: impl Into<TeamID>) -> Result<Team> {
        let id = id.into();
        let conn = &self.0;
        let team = conn.query_row("SELECT * FROM teams WHERE id = ?1;", params![id], |row| {
            Team::try_from(row)
        })?;
        Ok(team)
    }

    pub fn add_h2h(&self, h2h: &Head2Head) -> Result<()> {
        let conn = &self.0;
        conn.execute(
            "INSERT OR IGNORE INTO H2H (awayID, homeID, totalGames, teamWins, teamWinFreq) values (?1, ?2, ?3, ?4, ?5);",
            params![h2h.team1,h2h.team2,h2h.total_games,h2h.team_wins,h2h.team_win_freq],
        )?;
        Ok(())
    }

    pub fn get_h2h(&self, team1: impl Into<TeamID>, team2: impl Into<TeamID>) -> Result<Head2Head> {
        let id1 = team1.into();
        let id2 = team2.into();
        let conn = &self.0;
        let h2h = conn.query_row(
            "SELECT * FROM H2H WHERE awayID = ?1 AND homeID = ?2;",
            params![id1, id2],
            |row| Head2Head::try_from(row),
        )?;
        Ok(h2h)
    }

    pub fn add_last10(&self, id: impl Into<TeamID>) -> Result<()> {
        let id = id.into();
        let conn = &self.0;
        conn.execute(
            "INSERT OR IGNORE INTO last10 (id) VALUES (?1);",
            params![id],
        )?;
        Ok(())
    }

    pub fn get_last10(&self, id: impl Into<TeamID>) -> Result<Last10> {
        let id = id.into();
        let conn = &self.0;
        let last10 = conn.query_row("SELECT * FROM last10 WHERE id = ?1;", params![id], |row| {
            Last10::try_from(row)
        })?;
        Ok(last10)
    }

    pub fn get_team_abbrev(&self, abbrev: &str) -> Result<Team> {
        let conn = &self.0;
        let team = conn.query_row(
            "SELECT * FROM teams WHERE abbreviation = ?1;",
            params![abbrev],
            |row| Team::try_from(row),
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
            "UPDATE teams SET rating = ?1, uncertainty = ?2 WHERE id = ?3;",
            params![new_rating.rating, new_rating.uncertainty, id],
        )?;
        Ok(())
    }

    pub fn update_h2h(&self, h2h: Head2Head) -> Result<()> {
        let conn = &self.0;
        conn.execute(
            "UPDATE H2H SET totalGames = ?1, teamWins = ?2, teamWinFreq = ?3 WHERE awayID = ?4 AND homeID = ?5;",
            params![h2h.total_games,h2h.team_wins,h2h.team_win_freq,h2h.team1,h2h.team2]
        )?;
        Ok(())
    }

    pub fn update_last10(&self, last10: Last10) -> Result<()> {
        let conn = &self.0;
        let games = last10.games;
        let mut games_num = 0;
        for (idx, game) in games.iter().rev().enumerate() {
            let i = if *game == Outcomes::WIN { 1 } else { 0 };
            games_num |= i << idx;
        }
        conn.execute(
            "
        UPDATE last10
        SET wins = ?2, losses = ?3, games = ?4
        WHERE id = ?1;
        ",
            params![last10.id, last10.wins, last10.loss, games_num],
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
