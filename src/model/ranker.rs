use skillratings::{
    Outcomes,
    weng_lin::{self, weng_lin, weng_lin_two_teams},
};

use crate::{
    data::{
        db::{DataBase, TeamID},
        models::{prediction::Prediction, probability::DiscreteProb, teams::Team},
    },
    model::model::{Model, ModelBase},
    rating::openskill::RATING_CONFIG,
    utils::outcome_from_prob,
};

// const EPSILON: f64 = 1e-5;

// #[derive(Debug, Clone, Copy)]
// pub struct RankingModel<'a> {
//     db: &'a DataBase,
// }

pub type RankingModel<'a> = ModelBase<'a, [usize; 10001]>;

impl<'a> From<&'a DataBase> for RankingModel<'a> {
    fn from(db: &'a DataBase) -> Self {
        Self {
            db,
            dist: [0; 10001],
            succ: 0,
        }
    }
}

impl<'a> RankingModel<'a> {
    pub fn get_hits(&self) -> usize {
        self.dist.iter().sum()
    }

    pub fn get_prob(&self) -> DiscreteProb<10001> {
        let mut pmf = [0.; 10001];
        let mut cdf = [0.; 10001];
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

impl<'a> Model<Team> for RankingModel<'a> {
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
    ) -> rusqlite::Result<(Team, Team, Prediction)> {
        let id1 = away.into();
        let id2 = home.into();
        let team1 = self.db.get_team(id1)?;
        let team2 = self.db.get_team(id2)?;
        let (exp_away, exp_home) =
            weng_lin::expected_score(&team1.rating, &team2.rating, &RATING_CONFIG);
        let outcome = outcome_from_prob(exp_away, exp_home);
        Ok((
            team1,
            team2,
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
        let (mut away, mut home, predic) = self.predict_and_get(away, home)?;
        let (new_rank1, new_rank2) = weng_lin(&away.rating, &home.rating, &outcome, &RATING_CONFIG);
        away.update(new_rank1);
        home.update(new_rank2);
        self.db.update_team_rating(away.id, new_rank1)?;
        self.db.update_team_rating(home.id, new_rank2)?;
        let Prediction {
            exp_away, exp_home, ..
        } = &predic;
        let winexp = if let Outcomes::WIN = outcome {
            exp_away
        } else {
            exp_home
        };
        if predic.outcome == outcome {
            self.succ += 1;
        }
        let idx = self.exp2idx(*winexp);
        self.dist[idx] += 1;
        Ok(predic)
    }
}
