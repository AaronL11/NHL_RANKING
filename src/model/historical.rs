use log::debug;
use skillratings::Outcomes;

use crate::{
    data::{
        db::{DataBase, TeamID},
        models::{
            head2head::Head2Head,
            prediction::Prediction,
            probability::DiscreteProb,
            teams::{self, Team},
        },
    },
    model::model::{Model, ModelBase},
    utils::outcome_from_prob,
};

pub type HistoricalMatchupModel<'a> = ModelBase<'a, [usize; 1001]>;

impl<'a> From<&'a DataBase> for HistoricalMatchupModel<'a> {
    fn from(db: &'a DataBase) -> Self {
        Self {
            db,
            dist: [0; 1001],
            succ: 0,
        }
    }
}

impl<'a> HistoricalMatchupModel<'a> {
    pub fn get_hits(&self) -> usize {
        self.dist.iter().sum()
    }

    pub fn get_prob(&self) -> DiscreteProb<1001> {
        let mut pmf = [0.; 1001];
        let mut cdf = [0.; 1001];
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

impl<'a> Model<Head2Head> for HistoricalMatchupModel<'a> {
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
    ) -> rusqlite::Result<(Head2Head, Head2Head, Prediction)> {
        let id1 = away.into();
        let id2 = home.into();
        let h2h_away = self.db.get_h2h(id1, id2)?;
        let h2h_home = self.db.get_h2h(id2, id1)?;
        let (exp_away, exp_home) = (h2h_away.team_win_freq, h2h_home.team_win_freq);
        let outcome = outcome_from_prob(exp_away, exp_home);
        Ok((
            h2h_away,
            h2h_home,
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
        let (mut h2h_away, mut h2h_home, predic) = self.predict_and_get(away, home)?;
        // debug!("{h2h_away:#?}");
        let winexp = if let Outcomes::WIN = outcome {
            h2h_away.update(Outcomes::WIN);
            h2h_home.update(Outcomes::LOSS);
            &predic.exp_away
        } else {
            h2h_away.update(Outcomes::LOSS);
            h2h_home.update(Outcomes::WIN);
            &predic.exp_home
        };
        // debug!("{h2h_away:#?}");
        self.db.update_h2h(h2h_away)?;
        self.db.update_h2h(h2h_home)?;
        if predic.outcome == outcome {
            self.succ += 1;
        }
        let idx = (winexp * (self.dist.len() as f64 - 1.)).round() as usize;
        self.dist[idx] += 1;
        Ok(predic)
    }
}
