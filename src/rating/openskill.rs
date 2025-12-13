use skillratings::weng_lin::WengLinRating;

const DEFMMR: f64 = 1000.0;
const K: f64 = 3.0;
const SCALE: f64 = 40.0;

pub trait SkillRating {
    fn mmr(&self) -> i32;
}

impl SkillRating for WengLinRating {
    fn mmr(&self) -> i32 {
        rating_to_mmr(self.rating, self.uncertainty)
    }
}

pub fn rating_to_mmr(mu: f64, sigma: f64) -> i32 {
    std::cmp::max(0, (DEFMMR + (mu - K * sigma) * SCALE).round() as i32)
}
