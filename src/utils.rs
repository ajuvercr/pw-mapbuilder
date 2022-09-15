#[cfg(not(target_family = "wasm"))]
pub mod rng {

    use rand::rngs::StdRng;
    use rand::SeedableRng;
    pub type RNG = StdRng;

    pub fn new() -> RNG {
        StdRng::from_entropy()
    }
}

#[cfg(target_family = "wasm")]
pub mod rng {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    pub type RNG = StdRng;

    pub fn new() -> RNG {
        StdRng::from_entropy()

    }
}
