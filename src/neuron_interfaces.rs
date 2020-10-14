use std::rc::Rc;
use std::cell::RefCell;
use super::neuron::SensoryNeuron;

/// A sensory interface is an interface between
/// an analog sensor with a defined max and min value
/// and a sensory neuron
struct SensoryInterface {
    max: f32,
    min: f32,
    current_freq: Option<f32>,
    sensory_neuron: Rc<SensoryNeuron>
}

impl SensoryInterface {
    fn send_input_to_neuron(&mut self, input: f32) {
        let freq = (input - &self.min) / (&self.max - &self.min);
        if let Some(curr_freq) = self.current_freq {
            if curr_freq == freq {
                return;
            }
        }
        let alpha = self.sensory_neuron.alpha;
        let period = (((1. - (alpha / freq)).ln() / (1. - alpha).ln()) + 1.).round() as u32;
        self.sensory_neuron.set_period(period);
        self.current_freq = Some(freq);
    }
}


///
struct ActuatorInterface {
    max: f32,
    min: f32,
    output: RefCell<f32>
}