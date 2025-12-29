use serde::{Deserialize, Serialize};

use crate::data::models::prediction::Prediction;

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub rank_score: f64,
    pub hist_score: f64,
    pub la10_score: f64,
    pub outcome: u8,
}

pub struct DataPackage {
    pub away_data: Data,
    pub home_data: Data,
}

impl DataPackage {
    pub fn new(predictions: [Prediction; 3], outcome: u8) -> Self {
        let [rank, hist, la10] = predictions;
        let away_data = Data {
            rank_score: rank.exp_away,
            hist_score: hist.exp_away,
            la10_score: la10.exp_away,
            outcome,
        };
        let home_data = Data {
            rank_score: rank.exp_home,
            hist_score: hist.exp_home,
            la10_score: la10.exp_home,
            outcome: outcome ^ 1,
        };
        DataPackage {
            away_data,
            home_data,
        }
    }

    pub fn serialize(&self) -> SerializableDataPackage {
        SerializableDataPackage::new(
            self.away_data.rank_score,
            self.home_data.rank_score,
            self.away_data.hist_score,
            self.home_data.hist_score,
            self.away_data.la10_score,
            self.home_data.la10_score,
            self.away_data.outcome,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableDataPackage {
    pub away_rank: f64,
    pub home_rank: f64,
    pub away_hist: f64,
    pub home_hist: f64,
    pub away_la10: f64,
    pub home_la10: f64,
    pub outcome: u8,
}

impl SerializableDataPackage {
    fn new(
        away_rank: f64,
        home_rank: f64,
        away_hist: f64,
        home_hist: f64,
        away_la10: f64,
        home_la10: f64,
        outcome: u8,
    ) -> Self {
        Self {
            away_rank,
            home_rank,
            away_hist,
            home_hist,
            away_la10,
            home_la10,
            outcome,
        }
    }
}

impl From<SerializableDataPackage> for DataPackage {
    fn from(serdatapack: SerializableDataPackage) -> Self {
        Self {
            away_data: Data {
                rank_score: serdatapack.away_rank,
                hist_score: serdatapack.away_hist,
                la10_score: serdatapack.away_la10,
                outcome: serdatapack.outcome,
            },
            home_data: Data {
                rank_score: serdatapack.home_rank,
                hist_score: serdatapack.home_hist,
                la10_score: serdatapack.home_la10,
                outcome: serdatapack.outcome ^ 1,
            },
        }
    }
}
