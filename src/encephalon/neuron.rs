use std::rc::Rc;
use std::cell::RefCell;

pub struct Neuron {
    internal_charge: RefCell<f32>,
    threshold: f32,
    just_fired: bool,
    axon_synapses: Vec<Synapse>,
}

impl Neuron {
    pub fn new() -> Neuron {
        Neuron {
            internal_charge: RefCell::new(0.),
            threshold: 10.,
            just_fired: false,
            axon_synapses: Vec::new()
        }
    }

    pub fn create_synapse(&mut self, target_neuron: &Rc<Neuron>) {
        self.axon_synapses.push(
            Synapse::new(Rc::clone(target_neuron), SynapticType::Excitatory));
    }

    pub fn run_cycle(&mut self) {
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