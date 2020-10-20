use super::actuator::Actuator;
use super::neuron::SensoryNeuron;
use crate::neuron::ActuatorNeuron;
use crate::sensor::Sensor;
use std::rc::Rc;

/// This is an interface between an analog
/// sensor and its corresponding sensory
pub struct SensoryInterface {
    sensor: Rc<dyn Sensor>,
    pub sensory_neuron: Rc<SensoryNeuron>,
    encoder: fn(f32) -> u32,
}

impl SensoryInterface {
    pub fn new(
        sensor: Rc<dyn Sensor>,
        encoder: fn(f32) -> u32,
        sensory_neuron: Rc<SensoryNeuron>,
    ) -> SensoryInterface {
        SensoryInterface {
            sensor,
            encoder,
            sensory_neuron,
        }
    }

    /// Runs one encephalonaic cycle. Takes measurement
    /// from its sensor, encodes that measurement into
    /// a neuronic period, and sends that period to its
    /// sensory_neuron
    pub fn run_cycle(&self) {
        self.sensory_neuron
            .set_period((self.encoder)(self.sensor.measure()));
    }
}

pub mod sensory_encoders {
    /// This returns the period of a single pulsed time series
    /// that would result in "input" as the peak value of an
    /// exponential moving average (ema) over that interval
    ///
    /// Here alpha is the constant of the ema
    pub fn ema_encoder(measurement: f32, alpha: f32) -> u32 {
        (((1. - (alpha / measurement)).ln() / (1. - alpha).ln()) + 1.).round() as u32
    }

    /// This uses a linear function to decode sensory information.
    /// The linear function has a y intercept greater than 1, and
    /// contains the point (1, 1)
    pub fn linear_encoder(measurement: f32, y_int: f32) -> u32 {
        (((1. - y_int) * measurement) + y_int).round() as u32
    }

    /// This uses an inverse function (1/x) to decode sensory information
    /// This makes most sense for decoding
    pub fn inverse_encoder(measurement: f32) -> u32 {
        (1. / measurement).round() as u32
    }
}

/// This is the interface between an actuator neuron
/// and the actual actuator, which takes in an analog
/// value between min and max.  This interface essentially
/// provides the mechanism to translate between the neuron's
/// EMA and the actuator
pub struct ActuatorInterface {
    pub actuator_neuron: Rc<ActuatorNeuron>,
    actuator: Rc<dyn Actuator>,
}

impl ActuatorInterface {
    pub fn new(
        actuator_neuron: Rc<ActuatorNeuron>,
        actuator: Rc<dyn Actuator>,
    ) -> ActuatorInterface {
        ActuatorInterface {
            actuator_neuron,
            actuator,
        }
    }

    /// Runs one encephalonaic cycle. Measures its actuator
    /// neuron's (ema) frequency, and sets its actuator's
    /// control value to that frequency
    pub fn run_cycle(&self) {
        self.actuator
            .set_control_value(self.actuator_neuron.read_ema_frequency());
    }
}
