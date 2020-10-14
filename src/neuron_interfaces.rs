use std::rc::Rc;
use std::cell::RefCell;
use super::neuron::SensoryNeuron;
use sensory_encoders::Encoder;
use super::actuator::Actuator;

/// A sensory interface is an interface between
/// an analog sensor with a defined max and min value
/// and a sensory neuron
struct SensoryInterface {
    max: f32,
    min: f32,
    current_input: Option<f32>,
    encoder: Encoder,
    sensory_neuron: Rc<SensoryNeuron>
}

impl SensoryInterface {

    fn new(max: f32, min: f32, encoder: Encoder,
           sensory_neuron: Rc<SensoryNeuron>) -> SensoryInterface {
        SensoryInterface {
            max,
            min,
            current_input: None,
            encoder,
            sensory_neuron
        }
    }

    /// Does exactly what it says. Encodes its sensory
    /// input into the corresponding neuronic period and
    /// sends it to its corresponding
    fn send_input_to_neuron(&mut self, input: f32) {
        if let Some(curr_input) = self.current_input {
            if curr_input == input {
                return
            }
        }

        self.current_input = Some(input);
        let period = sensory_encoders::encode_sensory_input(input, self.encoder);
        self.sensory_neuron.set_period(period);
    }
}

mod sensory_encoders {
    /// This returns the period of a single pulsed time series
    /// that would result in "input" as the peak value of an
    /// exponential moving average (ema) over that interval
    ///
    /// Here alpha is the constant of the ema
    fn ema_encoder(input: f32, alpha: f32) -> u32 {
        (((1. - (alpha / input)).ln() / (1. - alpha).ln()) + 1.).round() as u32
    }

    /// This uses a linear function to decode sensory information.
    /// The linear function has a y intercept greater than 1, and
    /// contains the point (1, 1)
    fn linear_encoder(input: f32, y_int: f32) -> u32 {
        (((1. - y_int) * input) + y_int).round() as u32
    }

    /// This uses an inverse function (1/x) to decode sensory information
    /// This makes most sense for decoding
    fn inverse_encoder(input: f32) -> u32 {
        (1. / input).round() as u32
    }

    /// Enum describing all the different types of encoders possible,
    /// and has fields for their required constants
    #[derive(Copy, Clone)]
    pub enum Encoder {
        EMA(f32),
        Linear(f32),
        Inverse
    }

    /// Encodes the input using an encoding strategy
    pub fn encode_sensory_input(input: f32, encoder: Encoder) -> u32 {
        match encoder {
            Encoder::EMA(alpha) => ema_encoder(input, alpha),
            Encoder::Linear(y_int) => linear_encoder(input, y_int),
            Encoder::Inverse => inverse_encoder(input)
        }
    }
}


/// This is the interface between an actuator neuron
/// and the actual actuator, which takes in an analog
/// value between min and max.  This interface essentially
/// provides the mechanism to translate between the neuron's
/// EMA and the actuator
struct ActuatorInterface<'a, T: Actuator> {
    max: f32,
    min: f32,
    output: RefCell<f32>,
    actuator: &'a T
}

impl<'a, T: Actuator> ActuatorInterface<'a, T> {
    pub fn new(max: f32, min: f32, actuator: &'a T) -> ActuatorInterface<'a, T> {
        ActuatorInterface {
            max,
            min,
            output: RefCell::new(min),
            actuator
        }
    }

    /// This is called from the ActInt's actuator neuron
    /// which sets the output of the actuator based on the
    /// frequency of the neuron's firing.  The frequency is
    /// derived from a simple EMA
    pub fn set_output_from_freq(&self, freq: f32) {
        let output = (self.max - self.min) * freq + self.min;
        *self.output.borrow_mut() = output;
        self.actuator.set_value(output);
    }
}