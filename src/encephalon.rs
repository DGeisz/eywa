use crate::ecp_geometry::EcpGeometry;
use crate::neuron::{ChargeCycle, Neuronic, RxNeuronic};
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
    ecp_geometry: Box<dyn EcpGeometry>,
    rx_neurons: HashMap<String, Rc<dyn RxNeuronic>>,
    sensor_neurons: HashMap<String, Rc<dyn Neuronic>>,
}

impl Encephalon {
    /// Gets the elapsed cycle count of the encephalon.
    /// The cycle count dictates when sensor neurons fire,
    /// and also the ChargeCycle
    pub fn get_cycle_count(&self) -> u32 {
        *self.cycle_count.borrow() as u32
    }

    /// Indicates the parity of the charge cycle, which allows
    /// neurons to fire throughout a graphical structure without
    /// conflicting or incorrect behavior
    pub fn get_charge_cycle(&self) -> ChargeCycle {
        if *self.cycle_count.borrow() % 2 == 0 {
            ChargeCycle::Even
        } else {
            ChargeCycle::Odd
        }
    }

    /// Finds a random neuron within the vicinity of loc
    /// which allows neurons to make new random connections
    pub fn local_random_neuron(&self, loc: Vec<i32>) -> Option<Rc<dyn RxNeuronic>> {
        let hash_option = self.ecp_geometry.local_random_hash(loc);

        if let Some(hash) = hash_option {
            if let Some(rx_ref) = self.rx_neurons.get(&hash) {
                return Some(Rc::clone(rx_ref));
            }
        }

        None
    }
}
