use skillratings::Outcomes;

#[derive(Debug)]
pub struct Prediction {
    pub exp_away: f64,
    pub exp_home: f64,
    pub outcome: Outcomes,
}
