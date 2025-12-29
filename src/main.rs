#![allow(unused)]

mod data;
pub mod model;
mod rating;
mod utils;

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Write},
    time::Duration,
};

use crate::{
    data::{
        db::DataBase,
        models::{
            data::{Data, DataPackage, SerializableDataPackage},
            games::Game,
            teams::TeamsResponse,
        },
    },
    model::state::State,
    utils::in_season,
};
// use crate::rating::openskill::update_team_ratings;
use chrono::NaiveDate;
use env_logger::Env;
use itertools::{Either, Itertools};
use linfa::prelude::*;
use linfa_bayes::{GaussianNb, GaussianNbParams};
use linfa_logistic::LogisticRegression;
use linfa_trees::{DecisionTree, SplitQuality};
use log::{debug, info};
use ndarray::{Array1, Array2, ArrayView, array};
use ndarray_csv::Array2Reader;
use nhl_api::{Client, GameDate, GameType};
use plotters::{
    prelude::*,
    style::full_palette::{BLUE_400, RED_400, RED_900},
};
use rand::prelude::*;
use tokio::time::sleep;

const IMAGE_PATH_GIF: &'static str = "img/viz.gif";
const IMAGE_PATH: &'static str = "img/viz.png";
const DATA_PATH: &'static str = "data/metrics.csv";
const PATH: &'static str = "data/nhl_teams.db";
const _SAVE: &'static str = "standings";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start up the app
    // Start up the logger
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    // Start up the client
    let http = reqwest::Client::new();
    let client = Client::new()?;

    // let ranker = Ranker::new();

    // Start up the Database
    info!("Starting NHL Ranker...");

    info!("Fetching database");
    let db = DataBase::new(PATH)?;
    // db.clear()?;
    info!("Database started successfully");
    // info!("Fetching Dataset");
    // let mut ds = csv::Writer::from_path(DATA_PATH)?;
    // info!("Dataset fetched successfully");

    // info!("Adding Teams");
    // let mut ids = BTreeMap::new();
    // let game_week = client.weekly_schedule(None).await?.game_week;
    // for game_day in game_week {
    //     for game in game_day.games {
    //         let away = game.away_team;
    //         let home = game.home_team;
    //         ids.insert(away.abbrev, away.id);
    //         ids.insert(home.abbrev, home.id);
    //     }
    // }
    // let response = http
    //     .get("https://api.nhle.com/stats/rest/en/team")
    //     .send()
    //     .await?
    //     .json::<TeamsResponse>()
    //     .await?;
    // for team in &response.teams {
    //     db.add_team(team.id, team.full_name.clone(), team.tri_code.clone())?;
    //     db.add_last10(team.id)?;
    // }
    // info!("Teams added succesfully");

    // info!("Adding Head2Heads");
    // for team1 in &response.teams {
    //     let id1 = team1.id;
    //     for team2 in &response.teams {
    //         let id2 = team2.id;
    //         if id1 != id2 {
    //             let db_team1 = db.get_team(id1)?;
    //             let db_team2 = db.get_team(id2)?;
    //             let h2h = db_team1.vs(&db_team2);
    //             db.add_h2h(&h2h)?;
    //             let h2h = db_team2.vs(&db_team1);
    //             db.add_h2h(&h2h)?;
    //         }
    //     }
    // }
    // info!("Head2Heads added succesfully");

    // let mut state = State::from(&db);

    // let mut ngames = 0;
    // let (mut year, mut month, mut day) = (1955, 10, 1);
    // let (year_end, month_end, day_end) = (2025, 4, 20);
    // info!(
    //     "Retrieving daily scores between {month}-{day}-{year} till {month_end}-{day_end}-{year_end}"
    // );
    // print!("{ngames} games processed");
    // io::stdout().flush()?;
    // '_time_loop: while (year, month, day) != (year_end, month_end, day_end) {
    //     if year != 1955 && year % 5 == 0 && (month, day) == (1, 1) {
    //         debug!("Taking a break");
    //         sleep(Duration::from_mins(1)).await;
    //         debug!("Break done");
    //     }
    //     if !in_season(year, month, day) {
    //         month = 10;
    //         day = 1;
    //         continue;
    //     } else if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
    //         // debug!("Checking games on {date}");
    //         if let Ok(scores) = client.daily_scores(Some(GameDate::Date(date))).await {
    //             // debug!("Found {} games", scores.games.len());
    //             if scores.games.len() == 0 {
    //                 sleep(Duration::from_millis(100)).await;
    //             }
    //             'games_loop: for game in &scores.games {
    //                 if !game.game_state.is_final() {
    //                     continue;
    //                 }
    //                 if game.game_type != GameType::RegularSeason {
    //                     continue;
    //                 }
    //                 let predictions = state.process_game(&Game::from(game))?;
    //                 ngames += 1;
    //                 print!("\r{ngames} game(s) processed");
    //                 io::stdout().flush()?;
    //                 let outcome = if game.away_team.score > game.home_team.score {
    //                     1
    //                 } else {
    //                     0
    //                 };
    //                 let data_pack = DataPackage::new(predictions, outcome);
    //                 let DataPackage {
    //                     away_data,
    //                     home_data,
    //                 } = &data_pack;
    //                 for data in &[away_data, home_data] {
    //                     match data {
    //                         Data {
    //                             rank_score: 0.5, ..
    //                         } => continue 'games_loop,
    //                         Data {
    //                             hist_score: 0.0, ..
    //                         } => continue 'games_loop,
    //                         Data {
    //                             hist_score: 1.0, ..
    //                         } => continue 'games_loop,
    //                         _ => (),
    //                     }
    //                 }
    //                 ds.serialize(data_pack.serialize())?;
    //             }
    //         }
    //     }
    //     day += 1;
    //     if day > 31 {
    //         month += 1;
    //         day = 1;
    //     }
    //     if month > 12 {
    //         year += 1;
    //         month = 1;
    //     }
    // }
    // print!("\r");
    // io::stdout().flush()?;
    // info!("Daily scores retrieved! {ngames} processed");
    // let accs = state.get_accuracy();
    // let labels = vec!["ranking", "Head2Head", "Last 10 Games"];
    // for (label, (predicted_wins, ngames, acc)) in labels.iter().zip(accs.iter()) {
    //     println!(
    //         "The {} model predicted {} wins out of {} games with an accuracy of {:.2}%",
    //         label,
    //         predicted_wins,
    //         ngames,
    //         acc * 100.
    //     )
    // }

    // let n = 5;
    // info!("Querying the top and bottom {n} teams in the league");
    // let teams_mmr = db.get_top(n)?;
    // let teams = client
    //     .league_standings_for_date(&GameDate::Date(
    //         NaiveDate::from_ymd_opt(year, month, day).unwrap(),
    //     ))
    //     .await?;
    // println!("\nHere are the top {n} teams in the league");
    // let mut i = 1;
    // for (mmr_team, team) in teams_mmr.iter().zip(teams.iter()) {
    //     println!(
    //         "{} ({}) - {} ({})",
    //         team.team_name.default,
    //         i,
    //         mmr_team.name,
    //         mmr_team.rating.mmr(),
    //     );
    //     i += 1;
    // }
    // println!("\nHere are the bottom {n} teams in the league");
    // let teams_mmr = db.get_bot(n)?;
    // let mut i = 0;
    // for (mmr_team, team) in teams_mmr.iter().zip(teams.iter().rev()) {
    //     println!(
    //         "{} ({}) - {} ({})",
    //         team.team_name.default,
    //         32 - i,
    //         mmr_team.name,
    //         mmr_team.rating.mmr(),
    //     );
    //     i += 1;
    // }

    // println!("\nHere are the new league ratings:");
    // let teams_mmr = db.get_top(32)?;
    // let mut team_rank = HashMap::new();
    // let mut i = 1;
    // for mmr_team in &teams_mmr {
    //     println!("{i}: {} ({})", mmr_team.name, mmr_team.rating.mmr(),);
    //     team_rank.insert(mmr_team.name.clone(), (i, 0));
    //     i += 1;
    // }
    // // // i = 1;
    // // // for team in &teams {
    // // //     if let Some(rank) = team_rank.get_mut(&team.team_name.default) {
    // // //         let (mmr, _) = rank;
    // // //         *rank = (*mmr, i);
    // // //         i += 1;
    // // //     }
    // // // }
    // // // let mut new_teams = teams
    // // //     .iter()
    // // //     .map(|team| team.team_name.default.clone())
    // // //     .map(|team| {
    // // //         if team == "Utah Hockey Club" {
    // // //             String::from("Utah Mammoth")
    // // //         } else {
    // // //             team
    // // //         }
    // // //     })
    // // //     .collect::<Vec<_>>();
    // // // new_teams.sort_by(|t1, t2| {
    // // //     let (mmr1, stnd1) = team_rank[t1];
    // // //     let (mmr2, stnd2) = team_rank[t2];
    // // //     (stnd1 - mmr1).cmp(&(stnd2 - mmr2))
    // // // });

    // // println!("\nRanking by over/under rated");
    // // let mut i = 1;
    // // for team in new_teams {
    // //     let (mmr, stand) = team_rank[&team];
    // //     println!("{i} {} {}", team, (stand - mmr));
    // //     i += 1;
    // // }

    // // // Betting odds

    // sleep(Duration::from_mins(5)).await;
    // info!("Let's pick a winner for today");
    // let sched = client.daily_schedule(None).await?;
    // info!("Found {} games", sched.games.len());
    // let mut winners_rank = vec![];
    // let mut winners_freq = vec![];
    // for game in &sched.games {
    //     let away = &game.away_team;
    //     let home = &game.home_team;
    //     // let boxscore = client.boxscore(game.id).await?;
    //     // let boxscore = client.landing(game.id).await?;
    //     info!(
    //         "Game with id: {} will be played between {} @ {}",
    //         game.id,
    //         away.abbrev,
    //         // boxscore.away_team.common_name.default,
    //         // boxscore.home_team.common_name.default,
    //         home.abbrev,
    //     );
    //     let away_team = db.get_team_abbrev(&away.abbrev)?;
    //     let home_team = db.get_team_abbrev(&home.abbrev)?;
    //     info!(
    //         "The {} are rated: {}",
    //         &away_team.name,
    //         // away_team.rating.rating * 100.
    //         away_team.rating.mmr()
    //     );
    //     info!(
    //         "The {} are rated: {}",
    //         &home_team.name,
    //         // home_team.rating.rating * 100.
    //         home_team.rating.mmr()
    //     );
    //     let (exp1, exp2) =
    //         weng_lin::expected_score(&away_team.rating, &home_team.rating, &WengLinConfig::new());
    //     let h2h = db.get_h2h(away.id, home.id)?;
    //     let freq1 = h2h.team_win_freq;
    //     let freq2 = 1. - freq1;
    //     // let name_away = away_team.name.clone();
    //     // let name_away = away_team.name.clone();
    //     if exp1 > exp2 {
    //         info!(
    //             "{} is expected to win with probability {} by ranking",
    //             &away_team.name, exp1
    //         );
    //         winners_rank.push((away_team.name.clone(), exp1));
    //     } else {
    //         info!(
    //             "{} is expected to win with probability {} by ranking",
    //             &home_team.name, exp2
    //         );
    //         winners_rank.push((home_team.name.clone(), exp2))
    //     }
    //     if freq1 > freq2 {
    //         info!(
    //             "{} is expected to win with probability {} by ranking",
    //             &away_team.name, freq1
    //         );
    //         winners_freq.push((away_team.name.clone(), freq1));
    //     } else {
    //         info!(
    //             "{} is expected to win with probability {} by ranking",
    //             &home_team.name, exp2
    //         );
    //         winners_freq.push((home_team.name.clone(), freq2))
    //     }
    // }
    // winners_rank.sort_by(|(_, exp1), (_, exp2)| exp1.partial_cmp(&exp2).unwrap());
    // info!("Here are the expected winners for tonight's games by rank:");
    // for (team, exp) in winners_rank {
    //     println!("{} {:.2}%", team, exp * 100.);
    // }
    // winners_freq.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(&f2).unwrap());
    // info!("Here are the expected winners for tonight's games by history:");
    // for (team, f) in winners_freq {
    //     println!("{} {:.2}%", team, f * 100.);
    // }

    // Let's do some plotting

    let mut data = csv::Reader::from_path(DATA_PATH)?;

    let (wins, losses): (Vec<_>, Vec<_>) = data
        .records()
        .flatten()
        .flat_map(|x| x.deserialize::<Data>(None))
        .partition_map(|data| {
            if data.outcome == 1 {
                Either::Left(data)
            } else {
                Either::Right(data)
            }
        });

    // let root = BitMapBackend::gif(IMAGE_PATH_GIF, (640, 480), 50)?.into_drawing_area();
    // for pitch in 0..157 {
    //     // let pitch = 100;
    //     root.fill(&WHITE)?;
    //     let mut chart = ChartBuilder::on(&root)
    //         .caption(
    //             "Skill Rating - Historical Performance - Last 10 Games",
    //             ("sans-serif", 25).into_font(),
    //         )
    //         .margin(5)
    //         .x_label_area_size(30)
    //         .y_label_area_size(30)
    //         .build_cartesian_3d(0.0..1., 0.0..1., 0.0..1.)?;
    //     chart.with_projection(|mut p| {
    //         p.yaw = 1.57 - (1.57 - pitch as f64 / 50.0).abs();
    //         p.pitch = 1.57 - (1.57 - pitch as f64 / 50.0).abs();
    //         p.scale = 0.7;
    //         p.into_matrix() // build the projection matrix
    //     });

    //     chart.configure_axes().draw()?;

    //     let series = [
    //         (&wins, &RED_900, &RED_400, "wins"),
    //         (&losses, &BLUE, &BLUE_400, "losses"),
    //     ];
    //     for (datas, colour1, colour2, label) in series {
    //         chart
    //             .draw_series(
    //                 datas
    //                     .iter()
    //                     .map(
    //                         |Data {
    //                              rank_score,
    //                              hist_score,
    //                              la10_score,
    //                              ..
    //                          }| {
    //                             (rank_score, hist_score, la10_score)
    //                         },
    //                     )
    //                     .map(|(rank, hist, la10)| {
    //                         // let (rank, hist, la10) = (rank.clone(), hist.clone(), la10.clone());
    //                         EmptyElement::at((*rank, *hist, *la10))
    //                             // + Pixel::new((0, 0), ShapeStyle::from(&colour1).filled())
    //                         + Circle::new((0, 0), 1, ShapeStyle::from(&colour1).filled())
    //                         + Circle::new((0, 0), 0.5, ShapeStyle::from(&colour2).filled())
    //                     }),
    //             )?
    //             .label(label)
    //             .legend(|(x, y)| Circle::new((x, y), 3, ShapeStyle::from(&*colour2).filled()));
    //     }
    //     chart
    //         .configure_series_labels()
    //         .position(SeriesLabelPosition::UpperRight)
    //         .margin(20)
    //         .legend_area_size(5)
    //         .background_style(&WHITE.mix(0.8))
    //         .border_style(&BLACK)
    //         .draw()?;

    //     root.present()?;
    // }
    // root.present()?;

    // Let's do some learning

    info!("Fetching dataset for training");

    let data_file = File::open(DATA_PATH)?;
    let mut data = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(data_file);

    let mut records = vec![];
    let mut targets = vec![];
    let mut rows = 0;
    for DataPackage {
        away_data,
        home_data,
    } in data
        .deserialize::<SerializableDataPackage>()
        .flatten()
        .map(DataPackage::from)
    {
        records.append(&mut vec![
            away_data.rank_score,
            home_data.rank_score,
            away_data.hist_score,
            home_data.hist_score,
            away_data.la10_score,
            home_data.la10_score,
        ]);
        targets.push(away_data.outcome as usize);
        rows += 1;
    }
    let records = Array2::from_shape_vec((rows, 6), records)?;
    let targets = Array1::from_vec(targets);
    // let dataset = DatasetBase::new(records, targets);
    let dataset = DatasetBase::new(records, targets);
    let dataset = dataset
        .with_feature_names(vec![
            "Away Rating",
            "Home Rating",
            "Away Historical Record",
            "Home Historical Record",
            "Away Last 10 Games",
            "Home Last 10 Games",
        ])
        .with_target_names(vec!["Outcome"]);
    info!("Dataset fetched. Time to learn!");

    info!("Constructing Decision Tree");
    let tree_params = DecisionTree::params().split_quality(SplitQuality::Entropy);
    // dataset.

    info!("Tree constructed. Splitting dataset.");
    let mut rng = rand::thread_rng();
    let num_samples = 20_000;
    let mut boot = dataset.bootstrap_samples(num_samples, &mut rng);
    let boot_data = boot.next().unwrap();
    let mut train = boot.next().unwrap();
    let val = boot.next().unwrap();
    // let (mut train, val) = boot_data.split_with_ratio(0.9);
    // dataset.bootstrap()
    info!("Learning");
    let tree = tree_params.fit(&train)?;
    let logist = LogisticRegression::default();
    let logist = logist.fit(&train)?;
    let bayes = GaussianNbParams::new();
    let bayes = bayes.fit(&train)?;
    let mut boot_data = boot.next().unwrap();
    debug!("Trying Random Forest");
    let random_forest = boot
        .take(10)
        .flat_map(|booted_data| tree_params.fit(&booted_data))
        .collect::<Vec<_>>();
    // let random_forest = boot_data
    //     .iter_fold(10, |v| tree_params.fit(v))
    //     .map(|(t, _)| t.unwrap())
    //     .collect::<Vec<_>>();
    let mut predicted = Array1::from_vec(vec![(0, 0); val.targets.len()]);
    // random_forest.iter().fold(Array1::zeros((val.targets().shape())),|predicted,t| {
    //     t.predict(&val)
    // })
    debug!("Random Forest done!");
    for t in random_forest {
        let t_predicted = t.predict(&val);
        predicted
            .rows_mut()
            .into_iter()
            .flatten()
            .into_iter()
            .zip(t_predicted.rows().into_iter().flatten())
            .for_each(|((good, bad), pred)| if *pred == 1 { *good += 1 } else { *bad += 1 })
    }
    let predicted = Array1::from_iter(
        predicted
            .rows()
            .into_iter()
            .flatten()
            .map(|(good, bad)| if good > bad { 1 } else { 0 }),
    );
    //  rand::rng();
    info!("Time to predict!");
    let confusion_matrix: ConfusionMatrix<usize> = predicted.confusion_matrix(&val)?;
    println!(
        "Learning completed for Random Forest:\n\tRecall:\t\t{}\n\tAccuracy:\t {}\n\tPrecision:\t{}\n{:?}",
        confusion_matrix.recall(),
        confusion_matrix.accuracy(),
        confusion_matrix.precision(),
        confusion_matrix
    );
    let confusion_matrix = tree.predict(&val).confusion_matrix(&val)?;
    println!(
        "Learning completed for Decision Tree:\n\tRecall:\t\t{}\n\tAccuracy:\t {}\n\tPrecision:\t{}\n{:?}",
        confusion_matrix.recall(),
        confusion_matrix.accuracy(),
        confusion_matrix.precision(),
        confusion_matrix
    );
    let confusion_matrix = logist.predict(&val).confusion_matrix(&val)?;
    println!(
        "Learning completed for Logistic Regression:\n\tRecall:\t\t{}\n\tAccuracy:\t {}\n\tPrecision:\t{}\n{:?}",
        confusion_matrix.recall(),
        confusion_matrix.accuracy(),
        confusion_matrix.precision(),
        confusion_matrix
    );
    let confusion_matrix = bayes.predict(&val).confusion_matrix(&val)?;
    println!(
        "Learning completed for Naive Bayes:\n\tRecall:\t\t{}\n\tAccuracy:\t {}\n\tPrecision:\t{}\n{:?}",
        confusion_matrix.recall(),
        confusion_matrix.accuracy(),
        confusion_matrix.precision(),
        confusion_matrix
    );
    // let tree = DecisionTree::params();

    // clear up the database just in case
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
