use skillratings::Outcomes;

use crate::{
    data::{
        db::DataBase,
        models::{games::Game, prediction::Prediction},
    },
    model::{
        historical::HistoricalMatchupModel, last10::Last10GamesModel, model::Model,
        ranker::RankingModel,
    },
};

pub struct State<'a> {
    db: &'a DataBase,
    ranker: RankingModel<'a>,
    hist: HistoricalMatchupModel<'a>,
    last10: Last10GamesModel<'a>,
    pub dist: Vec<Vec<Vec<usize>>>,
    pub ngames: usize,
}

impl<'a> From<&'a DataBase> for State<'a> {
    fn from(db: &'a DataBase) -> Self {
        Self {
            db: db,
            ranker: RankingModel::from(db),
            hist: HistoricalMatchupModel::from(db),
            last10: Last10GamesModel::from(db),
            dist: vec![vec![vec![0; 11]; 1001]; 10001],
            ngames: 0,
        }
    }
}

impl<'a> State<'a> {
    pub fn process_game(&mut self, game: &Game) -> rusqlite::Result<[Prediction; 3]> {
        self.ngames += 1;
        let (away, home) = game.ids();
        let (away_score, home_score) = game.score;
        let outcome = if away_score > home_score {
            Outcomes::WIN
        } else {
            Outcomes::LOSS
        };
        let pred1 = self.ranker.predict_and_update(away, home, outcome)?;
        let pred2 = self.hist.predict_and_update(away, home, outcome)?;
        let pred3 = self.last10.predict_and_update(away, home, outcome)?;
        let idxr = if pred1.exp_away > pred1.exp_home {
            self.ranker.exp2idx(pred1.exp_away)
        } else {
            self.ranker.exp2idx(pred1.exp_home)
        };
        let idxh = if pred2.exp_away > pred1.exp_home {
            self.hist.exp2idx(pred1.exp_away)
        } else {
            self.hist.exp2idx(pred1.exp_home)
        };
        let idx10 = if pred3.exp_away > pred1.exp_home {
            self.last10.exp2idx(pred1.exp_away)
        } else {
            self.last10.exp2idx(pred1.exp_home)
        };
        self.dist[idxr][idxh][idx10] += 1;
        Ok([pred1, pred2, pred3])
    }

    pub fn process_games<'b>(
        &mut self,
        games: impl Iterator<Item = &'b Game>,
    ) -> rusqlite::Result<()> {
        for game in games {
            let _ = self.process_game(game)?;
        }
        Ok(())
    }

    pub fn get_accuracy(&self) -> Vec<(usize, usize, f64)> {
        let mut acc = vec![];
        let h1 = self.ranker.succ;
        let acc1 = h1 as f64 / self.ngames as f64;
        acc.push((h1, self.ngames, acc1));
        let h2 = self.hist.succ;
        let acc2 = h2 as f64 / self.ngames as f64;
        acc.push((h2, self.ngames, acc2));
        let h3 = self.last10.succ;
        let acc3 = h3 as f64 / self.ngames as f64;
        acc.push((h3, self.ngames, acc3));
        acc
    }
}
