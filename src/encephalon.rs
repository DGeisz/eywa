use super::neuron::ChargeCycle;
use std::cell::RefCell;

/// This is the brains of the operation (lol).
/// But, for real, this is contains a cluster of
/// primarily plastic neurons, with sensory, actuator,
/// and reflex neurons dancing around the edges.
///
/// This structure holds the global cycle count, which is
/// used by sensory neurons for determining when to fire,
/// and generally provides information about the extent to
/// which information hath traversed the encephalon
pub struct Encephalon {
    cycle_count: RefCell<u64>,
}

impl Encephalon {
    pub fn get_cycle_count(&self) -> u32 {
        *self.cycle_count.borrow() as u32
    }

    pub fn get_charge_cycle(&self) -> ChargeCycle {
        if *self.cycle_count.borrow() % 2 == 0 {
            ChargeCycle::Even
        } else {
            ChargeCycle::Odd
        }
    }
}
