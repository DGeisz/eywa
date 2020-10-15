use std::cell::RefCell;
use super::{RxNeuron, RxNeuronic};

/// All synapses have the capability to fire
pub trait Synapse {
    fn fire(&self);
}

/// This is a synapse that changes in strength
/// over time depending on the extent to which
/// it's firing it correlated with it's targets
/// firing. Can be excitatory or inhibitory.  If
/// this synapse strength passes beneath its
/// weakness threshold, it dissolves
pub struct PlasticSynapse {
    pub strength: RefCell<f32>,
    pub weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
    max_impulse: f32,
    growth_parameter: f32, //Must be between 0 and 1
    decay_parameter: f32, //Must be between 0 and 1
    synaptic_type: SynapticType,
    pub target: RxNeuron
}

impl PlasticSynapse {
    pub fn strengthen(&self) {
        let mut strength = self.strength.borrow_mut();
        *strength += (self.max_impulse - *strength) * self.growth_parameter;
    }

    pub fn decay(&self) {
       *self.strength.borrow_mut() *= self.decay_parameter;
    }
}

impl Synapse for PlasticSynapse {
    fn fire(&self) {
        let impulse = *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32);
        match &self.target {
            RxNeuron::Plastic(neuron_ref) => (*neuron_ref).intake_synaptic_impulse(impulse),
            RxNeuron::Actuator(neuron_ref) => (*neuron_ref).intake_synaptic_impulse(impulse),
            RxNeuron::Reflex(neuron_ref) => (*neuron_ref).intake_synaptic_impulse(impulse),
        }
    }
}

/// Excitatory synapses increase their target
/// neuron's internal charge, inhibitory synapses
/// decrease their target neuron's internal charge
/// to prevent the neuron from firing
enum SynapticType {
    Excitatory,
    Inhibitory
}

impl SynapticType {
    fn get_synapse_modifier(&self) -> i8 {
        match self {
            Self::Excitatory => 1,
            Self::Inhibitory => -1,
        }
    }
}