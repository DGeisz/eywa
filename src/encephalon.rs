use crate::actuator::Actuator;
use crate::ecp_geometry::EcpGeometry;
use crate::neuron::{ChargeCycle, Neuronic, RxNeuronic, ActuatorNeuron, RxNeuron};
use crate::sensor::Sensor;
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::neuron_interfaces::{ActuatorInterface, SensoryInterface};

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
    actuator_interfaces: Vec<ActuatorInterface>,
    sensory_interfaces: Vec<SensoryInterface>
}

impl Encephalon {
    /// Creates a new encephalon
    pub fn new(
        ecp_geometry: Box<dyn EcpGeometry>,
        sensors: Vec<Box<dyn Sensor>>,
        actuators: Vec<Box<dyn Actuator>>,

        //Parameters for neurons
        fire_threshold: f32,
        ema_alpha: f32
    ) -> Encephalon {
        if ecp_geometry.get_num_sensor() != sensors.len() as u32 {
            panic!(
                "The number of sensors passed to the encephalon doesn't \
             match the number of sensor neuron positions within the specified ecp_geometry"
            );
        } else if ecp_geometry.get_num_actuator() != actuators.len() as u32 {
            panic!(
                "The number of actuators passed to the encephalon doesn't \
             match the number of actuator neuron positions within the specified ecp_geometry"
            );
        }

        let mut actuator_count = 0;
        let mut rx_neurons = HashMap::new();

        let (mut loc, mut hash, mut neuron_type) = ecp_geometry.first_rx_loc();
        match neuron_type {
            RxNeuron::Actuator => {
                rx_neurons.insert(hash, Rc::new(
                    ActuatorNeuron::new(
                        fire_threshold
                    )
                ));
            },
            RxNeuron::Plastic => {}
        };

        loop {}

        Encephalon {}
    }

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
    pub fn local_random_neuron(&self, loc: &Vec<i32>) -> Option<Rc<dyn RxNeuronic>> {
        let hash_option = self.ecp_geometry.local_random_hash(loc);

        if let Some(hash) = hash_option {
            if let Some(rx_ref) = self.rx_neurons.get(&hash) {
                return Some(Rc::clone(rx_ref));
            }
        }

        None
    }
}
