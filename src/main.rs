mod data;
mod models;
mod rating;
mod utils;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use crate::{data::db::DataBase, rating::openskill::SkillRating};
// use crate::rating::openskill::update_team_ratings;
use chrono::{NaiveDate, Utc};
use env_logger::Env;
use log::{debug, info};
use nhl_api::{Client, GameDate};
use skillratings::{
    Outcomes,
    weng_lin::{self, WengLinConfig, weng_lin},
};

const PATH: &'static str = "src/data/nhl.db";
const SAVE: &'static str = "standings";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start up the app
    // Start up the logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // Start up the client
    let client = Client::new()?;

    // Start up the Database
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

    info!("Retrieving daily scores");
    let mut predicted_wins = 0;
    let mut ngames = 0;
    let (mut year, mut month, mut day) = (2025, 10, 7);
    while (year, month, day) == (2025, 12, 13) {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let scores = client.daily_scores(Some(GameDate::Date(date))).await?;
            for game in &scores.games {
                ngames += 1;
                let boxscore = client.boxscore(game.id).await?;
                info!(
                    "Game with id: {} was played between the {} and the {} on {}:",
                    game.id,
                    boxscore.away_team.common_name.default,
                    boxscore.home_team.common_name.default,
                    date
                );
                let away = &boxscore.away_team;
                let home = &boxscore.home_team;
                let outcome = if away.score > home.score {
                    Outcomes::WIN
                } else {
                    Outcomes::LOSS
                };
                info!(
                    "The score was: {} {} - {} {}",
                    away.abbrev, away.score, home.abbrev, home.score,
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
                let (exp1, exp2) = weng_lin::expected_score(
                    &away_team.rating,
                    &home_team.rating,
                    &WengLinConfig::new(),
                );
                info!(
                    "{} was expected to win",
                    if exp1 > exp2 {
                        away_team.name
                    } else {
                        home_team.name
                    }
                );
                if exp1 > 0.5 && outcome == Outcomes::WIN {
                    predicted_wins += 1;
                } else if exp2 >= 0.5 && outcome == Outcomes::LOSS {
                    predicted_wins += 1;
                }
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
                    // away_team.rating.rating * 100.
                    away_team.rating.mmr()
                );
                info!(
                    "The {} are now rated: {}",
                    home_team.name,
                    // home_team.rating.rating * 100.
                    home_team.rating.mmr()
                );
            }
        }
        day += 1;
        if day > 31 {
            month += 1;
            day = 1;
        }
        if month > 12 {
            year += 1;
            month = 1;
        }
    }
    println!(
        "The model predicted {} wins out of {} games with an accuracy of {:.2}%",
        predicted_wins,
        ngames,
        (predicted_wins as f64) / (ngames as f64) * 100.
    );

    let n = 5;
    info!("Querying the top and bottom {n} teams in the league");
    let teams_mmr = db.get_top(n)?;
    let teams = client.current_league_standings().await?;
    println!("\nHere are the top {n} teams in the league");
    let mut i = 1;
    for (mmr_team, team) in teams_mmr.iter().zip(teams.iter()) {
        println!(
            "{} ({}) - {} ({})",
            team.team_name.default,
            i,
            mmr_team.name,
            mmr_team.rating.mmr(),
        );
        i += 1;
    }
    println!("\nHere are the bottom {n} teams in the league");
    let teams_mmr = db.get_bot(n)?;
    let mut i = 0;
    for (mmr_team, team) in teams_mmr.iter().zip(teams.iter().rev()) {
        println!(
            "{} ({}) - {} ({})",
            team.team_name.default,
            32 - i,
            mmr_team.name,
            mmr_team.rating.mmr(),
        );
        i += 1;
    }

    println!("\nHere are the new league ratings:");
    let teams_mmr = db.get_top(32)?;
    let mut team_rank = HashMap::new();
    let mut i = 1;
    for mmr_team in &teams_mmr {
        println!("{i}: {} ({})", mmr_team.name, mmr_team.rating.mmr(),);
        team_rank.insert(mmr_team.name.clone(), (i, 0));
        i += 1;
    }
    i = 1;
    for team in &teams {
        if let Some(rank) = team_rank.get_mut(&team.team_name.default) {
            let (mmr, _) = rank;
            *rank = (*mmr, i);
            i += 1;
        }
    }
    let mut new_teams = teams
        .iter()
        .map(|team| team.team_name.default.clone())
        .collect::<Vec<_>>();
    new_teams.sort_by(|t1, t2| {
        let (mmr1, stnd1) = team_rank[t1];
        let (mmr2, stnd2) = team_rank[t2];
        (stnd1 - mmr1).cmp(&(stnd2 - mmr2))
    });

    println!("\nRanking by over/under rated");
    let mut i = 1;
    for team in new_teams {
        let (mmr, stand) = team_rank[&team];
        println!("{i} {} {}", team, (stand - mmr));
        i += 1;
    }

    info!("Deleting for reuse");
    db.clear()?;
    info!("Succesfully deleted tables");
    // // Fetch today's schedule
    // let today = Utc::now().format("%Y-%m-%d").to_string();
    // let games = fetch_games_for_date(&today).await?;
    // info!("Fetched {} games for {}", games.len(), today);

    // for game in games {
    //     // Create or retrieve both teams
    //     let home_team_id = get_or_create_team(&conn, &game.home_team)?;
    //     let away_team_id = get_or_create_team(&conn, &game.away_team)?;

    //     // Store game in DB
    //     insert_game_if_not_exists(&conn, &game, home_team_id, away_team_id)?;

    //     // Dummy example rating update (pretend home team wins)
    //     let (new_home, new_away) = update_team_ratings(
    //         vec![], // will eventually hold player ratings
    //         vec![],
    //         1.0, // home team wins
    //     );

    //     update_team_rating(&conn, home_team_id, new_home[0].mu)?;
    //     update_team_rating(&conn, away_team_id, new_away[0].mu)?;
    // }

    // info!("All games processed successfully.");
    Ok(())
}
