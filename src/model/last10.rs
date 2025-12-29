use std::collections::BTreeMap;

use log::debug;
use skillratings::Outcomes;

use crate::{
    data::{
        self,
        db::{DataBase, TeamID},
        models::{
            self,
            last10::Last10,
            prediction::{self, Prediction},
            probability::DiscreteProb,
        },
    },
    model::model::{Model, ModelBase},
};

pub type Last10GamesModel<'a> = ModelBase<'a, [usize; 11]>;

impl<'a> From<&'a DataBase> for Last10GamesModel<'a> {
    fn from(db: &'a DataBase) -> Self {
        Self {
            db,
            dist: [0; 11],
            succ: 0,
        }
    }
}

impl<'a> Last10GamesModel<'a> {
    pub fn get_hits(&self) -> usize {
        self.dist.iter().sum()
    }

    fn get_prob(&self) -> DiscreteProb<11> {
        let mut pmf = [0.; 11];
        let mut cdf = [0.; 11];
        let n = self.get_hits() as f64;
        for (i, x) in pmf.iter_mut().enumerate() {
            *x = self.dist[i] as f64 / n;
        }
        let mut sum = 0.;
        for (i, x) in cdf.iter_mut().enumerate() {
            sum += pmf[i];
            *x = sum;
        }
        cdf[cdf.len() - 1] = 1.;
        DiscreteProb { pmf, cdf }
    }

    pub fn exp2idx(&self, exp: f64) -> usize {
        (exp * (self.dist.len() as f64 - 1.)).round() as usize
    }
}

impl<'a> Model<Last10> for Last10GamesModel<'a> {
    fn predict(
        &self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
    ) -> rusqlite::Result<Prediction> {
        self.predict_and_get(away, home).map(|(_, _, pred)| pred)
    }

    fn predict_and_get(
        &self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
    ) -> rusqlite::Result<(Last10, Last10, Prediction)> {
        let id1 = away.into();
        let id2 = home.into();
        let away10 = self.db.get_last10(id1)?;
        let home10 = self.db.get_last10(id2)?;
        // debug!("{away10:#?}");
        let (exp_away, exp_home) = (away10.estimate(), home10.estimate());
        let outcome = if exp_away > exp_home {
            Outcomes::WIN
        } else {
            Outcomes::LOSS
        };
        Ok((
            away10,
            home10,
            Prediction {
                exp_away,
                exp_home,
                outcome,
            },
        ))
    }

    fn update(
        &mut self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
        outcome: Outcomes,
    ) -> rusqlite::Result<()> {
        self.predict_and_update(away, home, outcome).map(|_| ())
    }

    fn predict_and_update(
        &mut self,
        away: impl Into<TeamID>,
        home: impl Into<TeamID>,
        outcome: Outcomes,
    ) -> rusqlite::Result<Prediction> {
        let (mut away10, mut home10, prediction) = self.predict_and_get(away, home)?;
        let idx = if let Outcomes::WIN = outcome {
            away10.wins
        } else {
            home10.wins
        } as usize;
        self.dist[idx] += 1;
        away10.update(outcome);
        home10.update(outcome);
        self.db.update_last10(away10)?;
        self.db.update_last10(home10)?;
        if prediction.outcome == outcome {
            self.succ += 1;
        }
        Ok(prediction)
    }
}
