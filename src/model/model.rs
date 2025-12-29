use crate::{
    data::{
        db::{DataBase, TeamID},
        models::prediction::Prediction,
    },
    model::{historical::HistoricalMatchupModel, last10::Last10GamesModel, ranker::RankingModel},
};
use nhl_api::Client;
use skillratings::Outcomes;

#[derive(Debug, Clone, Copy)]
pub struct ModelBase<'a, T> {
    pub db: &'a DataBase,
    pub dist: T,
    pub succ: usize,
}

// impl<'a, T> From<&'a DataBase> for ModelBase<'a, T>
// where
//     T: Default,
// {
//     fn from(db: &'a DataBase) -> Self {
//         Self {
//             db,
//             dist: T::default(),
//         }
//     }
// }

pub trait Model<T> {
    fn predict(
        &self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
    ) -> rusqlite::Result<Prediction>;

    fn predict_and_get(
        &self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
    ) -> rusqlite::Result<(T, T, Prediction)>;

    fn update(
        &mut self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
        outcome: Outcomes,
    ) -> rusqlite::Result<()>;

    fn predict_and_update(
        &mut self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
        outcome: Outcomes,
    ) -> rusqlite::Result<Prediction>;
}
