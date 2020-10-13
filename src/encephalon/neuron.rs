use std::rc::Rc;
use std::cell::RefCell;

pub struct Neuron {
    internal_charge: RefCell<f32>,
    threshold: f32,
    just_fired: RefCell<bool>,
    axon_synapses: Vec<Synapse>
}

impl Neuron {
    pub fn new() -> Neuron {
        Neuron {
            internal_charge: RefCell::new(0.),
            threshold: 10.,
            just_fired: RefCell::new(false),
            axon_synapses: Vec::new()
        }
    }

    pub fn create_synapse(&mut self, target_neuron: &Rc<Neuron>) {
        self.axon_synapses.push(
            Synapse::new(Rc::clone(target_neuron), SynapticType::Excitatory));
    }

    pub fn attempt_fire(&self) {
        if *self.internal_charge.borrow() > self.threshold {
            println!("Yote");
        }
    }
}

struct Synapse {
    strength: f32,
    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
    synaptic_type: SynapticType,
    target: Rc<Neuron>
}

impl Synapse {
    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
        Synapse {
            strength: 1.,
            weakness_threshold: 0.1,
            synaptic_type,
            target
        }
    }
}

enum SynapticType {
    Excitatory,
    Inhibitory
}