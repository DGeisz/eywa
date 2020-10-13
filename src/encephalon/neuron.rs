use std::rc::Rc;
use std::cell::RefCell;
use rand::{Rng, random};

pub struct Neuron {
    neuron_type: NeuronType,

    internal_charge: RefCell<f32>,
    threshold: f32,
    just_fired: bool,

    max_axon_synapses: usize,
    axon_synapses: Vec<Synapse>,
}

pub enum NeuronType {
    Sensor(f32),
    Generic
}

impl Neuron {
    pub fn new(neuron_type: NeuronType) -> Neuron {
        Neuron {
            neuron_type,
            internal_charge: RefCell::new(0.),
            threshold: 10.,
            just_fired: false,
            max_axon_synapses: 20,
            axon_synapses: Vec::new()
        }
    }

    pub fn run_cycle(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
        self.prune_synapses();

        let mut internal_charge = self.internal_charge.borrow_mut();
        if *internal_charge > self.threshold {
            self.fire_synapses();

            self.just_fired = true;
            *internal_charge = 0.;
        } else {
            self.just_fired = false;
        }
    }

    fn create_random_synapse(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
        if self.axon_synapses.len() < self.max_axon_synapses {
            let random_index = rand::thread_rng().gen_range(0, neuron_pool.len());

            if let Some(neuron) = neuron_pool.get(random_index) {
                self.create_synapse(neuron);
            }
        }
    }

    pub fn create_synapse(&mut self, target_neuron: &Rc<Neuron>) {
        self.axon_synapses.push(
            Synapse::new(Rc::clone(target_neuron), SynapticType::Excitatory));
    }

    fn prune_synapses(&mut self) {
        let synapses = &mut self.axon_synapses;
        let synapses_fired = self.just_fired;

        synapses.retain(|synapse: &Synapse|{

            let mut strength = synapse.strength.borrow_mut();

            if synapses_fired {
                if synapse.target.just_fired {
                    *strength += (synapse.max_impulse - *strength) * synapse.growth_parameter;
                } else {
                    *strength *= synapse.decay_parameter;
                }
            }

            *strength > synapse.weakness_threshold
        })
    }

    fn fire_synapses(&self) {
        for synapse in &self.axon_synapses {
            synapse.fire();
        }
    }

    fn intake_synaptic_impulse(&self, charge: f32) {
        *self.internal_charge.borrow_mut() += charge;
    }
}


struct Synapse {
    strength: RefCell<f32>,
    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
    max_impulse: f32,
    growth_parameter: f32, //Must be between 0 and 1
    decay_parameter: f32, //Must be between 0 and 1
    synaptic_type: SynapticType,
    target: Rc<Neuron>
}

impl Synapse {
    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
        Synapse {
            strength: RefCell::new(1.),
            weakness_threshold: 0.1,
            max_impulse: 11.,
            growth_parameter: 0.1,
            decay_parameter: 0.1,
            synaptic_type,
            target
        }
    }

    fn fire(&self) {
        self.target.intake_synaptic_impulse(
            *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32))
    }
}

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