use std::cell::RefCell;
use std::rc::Rc;
use super::encephalon::Encephalon;

mod synapse;
use synapse::{PlasticSynapse, Synapse};

/// All neurons implement the Neuronic trait
pub trait Neuronic {
    fn run_cycle(&self);
    fn fire(&self);
}

/// Neurons that transmit (hence Tx) impulses to
/// to other neurons implement the TxNeuronic trait
pub trait TxNeuronic {
    fn fire_synapses(&self);
}

/// Enum of all different neurons that implement
/// the the trait RxNeuronic
pub enum TxNeuron<'a> {
    Sensory(Rc<SensoryNeuron<'a>>),
    Plastic(Rc<PlasticNeuron>),
    Reflex(Rc<ReflexNeuron>)
}

/// Neurons that receive (hence Rx) impulses from
/// other neurons implement the RxNeuronic trait
pub trait RxNeuronic {
    fn intake_synaptic_impulse(&self, impulse: f32);
}

/// Enum of all different neurons that implement
/// the the trait RxNeuronic
pub enum RxNeuron {
    Plastic(Rc<PlasticNeuron>),
    Actuator(Rc<ActuatorNeuron>),
    Reflex(Rc<ReflexNeuron>)
}

impl RxNeuron {

    /// Umbrella method to determine if the RxNeuron just
    /// fired or not
    fn did_just_fire(&self) -> bool {
        match self {
            Self::Plastic(neuron_ref) => (**neuron_ref).just_fired,
            Self::Actuator(neuron_ref) => (**neuron_ref).just_fired,
            Self::Reflex(neuron_ref) => (**neuron_ref).just_fired,
        }
    }
}

/// Here Fx stands for "flex" (don't confuse this with
/// Rx or Tx, it has nothing to do with transmission, I
/// just like the lexical symmetry).  Any neuron that displays
/// some level of plasticity implements FxNeuronic.
///
/// Here plasticity refers to neurons whose synapses strengthen,
/// weaken, dissolve, or appear over time
pub trait FxNeuronic {

    /// Strengthens or decays plastic synapses and dissolves
    /// synapses whose strength has fallen beneath it's
    /// weakness threshold
    fn prune_synapses(&mut self);

    /// Creates new synapse
    fn form_synapse(&mut self);
}

/// A neuron that sends encoded sensory information into
/// an encephalon
pub struct SensoryNeuron<'a> {
    encephalon: &'a Encephalon,
    period: RefCell<u32>, //This is the period at which the neuron fires
    axon_synapses: Vec<PlasticSynapse>,
    pub just_fired: bool,
}

impl<'a> SensoryNeuron<'a> {
    pub fn set_period(&self, period: u32) {
        *self.period.borrow_mut() = period;
    }
}

impl Neuronic for SensoryNeuron<'_> {
    fn run_cycle(&self) {
        if self.encephalon.get_cycle_count() % *self.period.borrow() == 0 {
            self.fire();
        }
    }

    fn fire(&self) {
        self.fire_synapses();
    }
}

impl TxNeuronic for SensoryNeuron<'_> {
    fn fire_synapses(&self) {
        for synapse in &self.axon_synapses {
            synapse.fire();
        }
    }
}

impl FxNeuronic for SensoryNeuron<'_> {
    fn prune_synapses(&mut self) {
        let synapses = &mut self.axon_synapses;
        let synapses_fired = self.just_fired;

        synapses.retain(|synapse| {
            if synapses_fired {
                if synapse.target.did_just_fire() {
                    synapse.strengthen();
                } else {
                    synapse.decay();
                }
            }
            let strength = synapse.strength.borrow();
            *strength > synapse.weakness_threshold
        })
    }

    fn form_synapse(&mut self) {

    }
}

/// A neuron that receives impulses but only
/// sends its average frequency (calculated via EMA)
/// to an ActuatorInterface
pub struct ActuatorNeuron {
   //TODO: Impl this
   pub just_fired: bool,
}

impl RxNeuronic for ActuatorNeuron {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        //TODO: Impl this
    }
}

/// This is a neuron that is essentially fixed in a
/// particular location, typically between a sensor neuron
/// and an actuator neuron
pub struct ReflexNeuron {
    //TODO: Determine if I even need this in the face of
    // Static synapses, and impl if so
    pub just_fired: bool,
}

impl RxNeuronic for ReflexNeuron {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        //TODO: Impl this
    }
}

/// This is your standard neuron present in the
/// encephalon.  Basically everything about this
/// neuron isn't fixed.  It's incoming or outgoing
/// synapses are subject to change based on its
/// environment
pub struct PlasticNeuron {
    pub just_fired: bool,
}

impl RxNeuronic for PlasticNeuron {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        //TODO: Impl this
    }
}



//struct Synapse {
//    strength: RefCell<f32>,
//    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
//    max_impulse: f32,
//    growth_parameter: f32, //Must be between 0 and 1
//    decay_parameter: f32, //Must be between 0 and 1
//    synaptic_type: SynapticType,
//    target: Rc<Neuron>
//}
//
//impl Synapse {
//    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
//        Synapse {
//            strength: RefCell::new(1.),
//            weakness_threshold: 0.1,
//            max_impulse: 11.,
//            growth_parameter: 0.1,
//            decay_parameter: 0.1,
//            synaptic_type,
//            target
//        }
//    }
//
//    fn fire(&self) {
//        self.target.intake_synaptic_impulse(
//            *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32))
//    }
//}
//
//enum SynapticType {
//    Excitatory,
//    Inhibitory
//}
//
//impl SynapticType {
//    fn get_synapse_modifier(&self) -> i8 {
//        match self {
//            Self::Excitatory => 1,
//            Self::Inhibitory => -1,
//        }
//    }
//}



//use std::rc::Rc;
//use std::cell::RefCell;
//use rand::{Rng, random};
//
//pub struct Neuron {
//    neuron_type: NeuronType,
//
//    internal_charge: RefCell<f32>,
//    threshold: f32,
//    just_fired: bool,
//
//    max_axon_synapses: usize,
//    axon_synapses: Vec<Synapse>,
//}
//
//pub enum NeuronType {
//    Sensor(f32),
//    Generic
//}
//
//impl Neuron {
//    pub fn new(neuron_type: NeuronType) -> Neuron {
//        Neuron {
//            neuron_type,
//            internal_charge: RefCell::new(0.),
//            threshold: 10.,
//            just_fired: false,
//            max_axon_synapses: 20,
//            axon_synapses: Vec::new()
//        }
//    }
//
//    pub fn run_cycle(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
//        self.prune_synapses();
//
//        let mut internal_charge = self.internal_charge.borrow_mut();
//        if *internal_charge > self.threshold {
//            self.fire_synapses();
//
//            self.just_fired = true;
//            *internal_charge = 0.;
//        } else {
//            self.just_fired = false;
//        }
//    }
//
//    fn create_random_synapse(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
//        if self.axon_synapses.len() < self.max_axon_synapses {
//            let random_index = rand::thread_rng().gen_range(0, neuron_pool.len());
//
//            if let Some(neuron) = neuron_pool.get(random_index) {
//                self.create_synapse(neuron);
//            }
//        }
//    }
//
//    pub fn create_synapse(&mut self, target_neuron: &Rc<Neuron>) {
//        self.axon_synapses.push(
//            Synapse::new(Rc::clone(target_neuron), SynapticType::Excitatory));
//    }
//
//    fn prune_synapses(&mut self) {
//        let synapses = &mut self.axon_synapses;
//        let synapses_fired = self.just_fired;
//
//        synapses.retain(|synapse: &Synapse| {
//
//            let mut strength = synapse.strength.borrow_mut();
//
//            if synapses_fired {
//                if synapse.target.just_fired {
//                    *strength += (synapse.max_impulse - *strength) * synapse.growth_parameter;
//                } else {
//                    *strength *= synapse.decay_parameter;
//                }
//            }
//
//            *strength > synapse.weakness_threshold
//        })
//    }
//
//    fn fire_synapses(&self) {
//        for synapse in &self.axon_synapses {
//            synapse.fire();
//        }
//    }
//
//    fn intake_synaptic_impulse(&self, charge: f32) {
//        *self.internal_charge.borrow_mut() += charge;
//    }
//}
//
//
//struct Synapse {
//    strength: RefCell<f32>,
//    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
//    max_impulse: f32,
//    growth_parameter: f32, //Must be between 0 and 1
//    decay_parameter: f32, //Must be between 0 and 1
//    synaptic_type: SynapticType,
//    target: Rc<Neuron>
//}
//
//impl Synapse {
//    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
//        Synapse {
//            strength: RefCell::new(1.),
//            weakness_threshold: 0.1,
//            max_impulse: 11.,
//            growth_parameter: 0.1,
//            decay_parameter: 0.1,
//            synaptic_type,
//            target
//        }
//    }
//
//    fn fire(&self) {
//        self.target.intake_synaptic_impulse(
//            *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32))
//    }
//}
//
//enum SynapticType {
//    Excitatory,
//    Inhibitory
//}
//
//impl SynapticType {
//    fn get_synapse_modifier(&self) -> i8 {
//        match self {
//            Self::Excitatory => 1,
//            Self::Inhibitory => -1,
//        }
//    }
//}
