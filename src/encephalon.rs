use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::actuator::Actuator;
use crate::ecp_geometry::EcpGeometry;
use crate::neuron::{ActuatorNeuron, ChargeCycle, Neuronic, RxNeuron, RxNeuronic, PlasticNeuron};
use crate::neuron::synapse::synaptic_strength::SynapticStrength;
use crate::neuron_interfaces::{ActuatorInterface, SensoryInterface};
use crate::sensor::Sensor;

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
    rx_neurons: RefCell<HashMap<String, Rc<dyn RxNeuronic>>>,
    sensor_neurons: RefCell<HashMap<String, Rc<dyn Neuronic>>>,
    actuator_interfaces: RefCell<HashMap<String, ActuatorInterface>>,
    sensory_interfaces: RefCell<HashMap<String, SensoryInterface>>,
}

impl Encephalon {
    /// Creates a new encephalon
    pub fn new(
        ecp_geometry: Box<dyn EcpGeometry>,
        sensors: Vec<Rc<dyn Sensor>>,
        actuators: Vec<Rc<dyn Actuator>>,

        //Parameters for neurons
        fire_threshold: f32,
        ema_alpha: f32,
        synaptic_strength_generator: fn() -> Box<RefCell<dyn SynapticStrength>>,
        synapse_type_threshold: f32,
        max_plastic_synapses: usize
    ) -> Rc<Encephalon> {
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

        let new_encephalon = Rc::new(Encephalon {
            cycle_count: RefCell::new(0),
            ecp_geometry,
            rx_neurons: RefCell::new(HashMap::new()),
            sensor_neurons: RefCell::new(HashMap::new()),
            actuator_interfaces: RefCell::new(HashMap::new()),
            sensory_interfaces: RefCell::new(HashMap::new()),
        });

        let mut actuator_count = 0;

        // let (mut loc, mut hash, mut neuron_type) = new_encephalon.ecp_geometry.first_rx_loc();
        let mut ecp_rx_option = Some(new_encephalon.ecp_geometry.first_rx_loc());

        loop {
            if let Some((loc, hash, neuron_type)) = &ecp_rx_option {
                match neuron_type {
                    RxNeuron::Actuator => {
                        let new_neuron = Rc::new(ActuatorNeuron::new(
                            Rc::clone(&new_encephalon),
                            fire_threshold,
                            ema_alpha,
                            loc.clone(),
                        ));

                        let new_rx_neuron = Rc::clone(&new_neuron);

                        new_encephalon
                            .rx_neurons
                            .borrow_mut()
                            .insert(hash.clone(), Rc::clone(&(new_rx_neuron as Rc<dyn RxNeuronic>)));

                        let curr_actuator_option = actuators.get(actuator_count);

                        if let Some(curr_actuator) = curr_actuator_option {
                            new_encephalon.actuator_interfaces.borrow_mut().insert(
                                curr_actuator.get_name(),
                                ActuatorInterface::new(Rc::clone(&new_neuron), Rc::clone(&curr_actuator)),
                            );
                        }

                        actuator_count += 1;
                    }
                    RxNeuron::Plastic => {
                        new_encephalon
                            .rx_neurons
                            .borrow_mut()
                            .insert(
                                hash.clone(),
                                Rc::new(PlasticNeuron::new(
                                    Rc::clone(&new_encephalon),
                                    fire_threshold,
                                    max_plastic_synapses,
                                    synaptic_strength_generator,
                                    synapse_type_threshold,
                                    ema_alpha,
                                    loc.clone(),
                                )));
                    }
                };

                ecp_rx_option = new_encephalon.ecp_geometry.next_rx_loc(loc.clone());
            } else {
                break;
            }
        }

        new_encephalon
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
            if let Some(rx_ref) = self.rx_neurons.borrow().get(&hash) {
                return Some(Rc::clone(rx_ref));
            }
        }

        None
    }
}
