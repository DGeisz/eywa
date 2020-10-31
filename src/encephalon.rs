use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;

use crate::actuator::Actuator;
use crate::ecp_geometry::EcpGeometry;
use crate::neuron::synapse::synaptic_strength::SynapticStrength;
use crate::neuron::synapse::SynapticType;
use crate::neuron::{
    ActuatorNeuron, ChargeCycle, Neuronic, NeuronicRx, PlasticNeuron, RxNeuron, SensoryNeuron,
    TxNeuronic,
};
use crate::neuron_interfaces::{ActuatorInterface, SensoryInterface};
use crate::sensor::Sensor;

/// This is a high level description of a reflex.
/// A reflex is a static synapse between a sensor
/// and actuator neuron of a fixed strength
pub struct Reflex {
    pub sensor_name: String,
    pub actuator_name: String,
    pub synapse_type: SynapticType,
    pub strength: f32,
}

impl Reflex {
    pub fn new(
        sensor_name: String,
        actuator_name: String,
        synapse_type: SynapticType,
        strength: f32,
    ) -> Reflex {
        Reflex {
            sensor_name,
            actuator_name,
            synapse_type,
            strength,
        }
    }
}

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
    rx_neurons: RefCell<HashMap<String, Rc<dyn NeuronicRx>>>,
    sensory_neurons: RefCell<HashMap<String, Rc<SensoryNeuron>>>,
    actuator_interfaces: RefCell<HashMap<String, ActuatorInterface>>,
    sensory_interfaces: RefCell<HashMap<String, SensoryInterface>>,
    reflexes: Vec<Reflex>,
}

impl Encephalon {
    /// Creates a new encephalon.
    pub fn new(
        ecp_geometry: Box<dyn EcpGeometry>,
        mut sensors: Vec<Box<dyn Sensor>>,
        mut actuators: Vec<Box<dyn Actuator>>,

        //Parameters for neurons
        fire_threshold: f32,
        ema_alpha: f32,
        synaptic_strength_generator: Rc<dyn Fn() -> Box<RefCell<dyn SynapticStrength>>>,
        synapse_type_threshold: f32,
        max_plastic_synapses: usize,

        //Parameters for interfaces
        sensory_encoder: fn(f32) -> u32,

        //List of reflex synapses
        reflexes: Vec<Reflex>,
    ) -> Rc<Encephalon> {
        if ecp_geometry.get_num_sensory() != sensors.len() as u32 {
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
            sensory_neurons: RefCell::new(HashMap::new()),
            actuator_interfaces: RefCell::new(HashMap::new()),
            sensory_interfaces: RefCell::new(HashMap::new()),
            reflexes,
        });

        // Populate the encephalon's Rx neurons
        let mut ecp_rx_option = Some(new_encephalon.ecp_geometry.first_rx_loc());

        loop {
            if let Some((loc, hash, neuron_type)) = &ecp_rx_option {
                match neuron_type {
                    RxNeuron::Actuator => {
                        // println!("Made actuator neuron!");
                        let new_neuron = Rc::new(ActuatorNeuron::new(
                            Rc::clone(&new_encephalon),
                            fire_threshold,
                            ema_alpha,
                        ));

                        let new_rx_neuron = Rc::clone(&new_neuron);

                        new_encephalon.rx_neurons.borrow_mut().insert(
                            hash.clone(),
                            Rc::clone(&(new_rx_neuron as Rc<dyn NeuronicRx>)),
                        );

                        let curr_actuator_option = actuators.pop();

                        if let Some(curr_actuator) = curr_actuator_option {
                            new_encephalon.actuator_interfaces.borrow_mut().insert(
                                curr_actuator.get_name(),
                                ActuatorInterface::new(Rc::clone(&new_neuron), curr_actuator),
                            );
                        }
                    }
                    RxNeuron::Plastic => {
                        // println!("Made plastic neuron!");
                        new_encephalon.rx_neurons.borrow_mut().insert(
                            hash.clone(),
                            Rc::new(PlasticNeuron::new(
                                Rc::clone(&new_encephalon),
                                fire_threshold,
                                max_plastic_synapses,
                                Rc::clone(&synaptic_strength_generator),
                                synapse_type_threshold,
                                ema_alpha,
                                loc.clone(),
                            )),
                        );
                    }
                };

                ecp_rx_option = new_encephalon.ecp_geometry.next_rx_loc(loc.clone());
            } else {
                break;
            }
        }

        // Populate the encephalon's sensory_neurons
        let mut ecp_sensory_option = Some(new_encephalon.ecp_geometry.first_sensory_loc());

        loop {
            if let Some((loc, hash)) = &ecp_sensory_option {
                let new_neuron = Rc::new(SensoryNeuron::new(
                    Rc::clone(&new_encephalon),
                    max_plastic_synapses,
                    Rc::clone(&synaptic_strength_generator),
                    synapse_type_threshold,
                    ema_alpha,
                    loc.clone(),
                ));

                new_encephalon
                    .sensory_neurons
                    .borrow_mut()
                    .insert(hash.clone(), Rc::clone(&new_neuron));

                let curr_sensor_option = sensors.pop();

                if let Some(curr_sensor) = curr_sensor_option {
                    new_encephalon.sensory_interfaces.borrow_mut().insert(
                        curr_sensor.get_name(),
                        SensoryInterface::new(curr_sensor, sensory_encoder, Rc::clone(&new_neuron)),
                    );
                }

                ecp_sensory_option = new_encephalon.ecp_geometry.next_sensory_loc(loc.clone());
            } else {
                break;
            }
        }

        new_encephalon.form_reflex_synapses();

        new_encephalon
    }

    /// Runs one full cycle of the encephalon
    pub fn run_cycle(&self) {
        self.uptick_cycle_count();

        // Cycle sensory interfaces
        for sensory_interface in self.sensory_interfaces.borrow_mut().values_mut() {
            sensory_interface.run_cycle();
        }

        // Cycle actuator interfaces
        for actuator_interface in self.actuator_interfaces.borrow().values() {
            actuator_interface.run_cycle();
        }

        // let mut sensor_ema_total: f32 = 0.0;

        // Cycle sensory neurons
        for sensory_neuron in self.sensory_neurons.borrow().values() {
            // sensor_ema_total += sensory_neuron.run_cycle();
            sensory_neuron.run_cycle();
        }

        // let sensor_ema_average = sensor_ema_total / self.sensory_neurons.borrow().len() as f32;

        // let mut rx_ema_total: f32 = 0.;

        // Cycle rx neurons
        for rx_neuron in self.rx_neurons.borrow().values() {
            // rx_ema_total += rx_neuron.run_cycle();
            rx_neuron.run_cycle();
        }

        // let rx_ema_average = rx_ema_total / self.rx_neurons.borrow().len() as f32;

        // println!("Sensor EMA: {}, Rx EMA: {}", sensor_ema_average, rx_ema_average);
    }

    /// Runs a certain number of full cycles
    pub fn run_n_cycles(&self, n: u32) {
        let mut start = SystemTime::now();

        for i in 0..n {
            self.run_cycle();

            if i % 100 == 0 {
                println!(
                    "Cycle: {} Elapsed time: {}",
                    i,
                    start.elapsed().unwrap().as_secs_f32()
                );
                start = SystemTime::now();
            }
        }
    }

    /// Upticks cycle count by 1
    fn uptick_cycle_count(&self) {
        *self.cycle_count.borrow_mut() += 1;
    }

    /// Forms static reflex synapses from the list
    /// of reflexes passed into Encephalon during creation
    fn form_reflex_synapses(&self) {
        for reflex in &self.reflexes {
            if let Some(sensor) = self.sensory_interfaces.borrow().get(&reflex.sensor_name) {
                if let Some(actuator) = self.actuator_interfaces.borrow().get(&reflex.actuator_name)
                {
                    sensor.sensory_neuron.add_static_synapse(
                        reflex.strength,
                        reflex.synapse_type,
                        Rc::clone(&(Rc::clone(&actuator.actuator_neuron) as Rc<dyn NeuronicRx>)),
                    );
                }
            }
        }
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
    pub fn local_random_neuron(&self, loc: &Vec<i32>) -> Option<Rc<dyn NeuronicRx>> {
        let hash_option = self.ecp_geometry.local_random_hash(loc);
        if let Some(hash) = hash_option {
            if let Some(rx_ref) = self.rx_neurons.borrow().get(&hash) {
                return Some(Rc::clone(rx_ref));
            }
        }
        None
    }
}
