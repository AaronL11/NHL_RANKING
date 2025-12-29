pub struct DiscreteProb<const N: usize> {
    pub pmf: [f64; N],
    pub cdf: [f64; N],
}
