use crate::{dds::*, AcceptFunction};

pub struct Simulation {
    pub n_deals: usize,
    pub accept_function: AcceptFunction,
}
