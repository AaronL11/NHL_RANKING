pub mod ids;

use skillratings::Outcomes;

pub const EPSILON: f64 = 1e-5;

pub fn outcome_from_prob(exp_away: f64, exp_home: f64) -> Outcomes {
    if (exp_away - exp_home).abs() < EPSILON {
        Outcomes::DRAW
    } else if exp_away > exp_home {
        Outcomes::WIN
    } else {
        Outcomes::LOSS
    }
}

pub fn in_season(_year: i32, month: u32, _day: u32) -> bool {
    month >= 10 || 4 >= month
}
