use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use eywa::ecp_geometry::{BoxEcp, EcpGeometry};
use eywa::encephalon::{Encephalon, Reflex};
use eywa::neuron::synapse::synaptic_strength::SigmoidStrength;
use eywa::neuron::synapse::SynapticType;
use eywa::neuron_interfaces::sensory_encoders;
use eywa::{Actuator, Sensor};

fn encoder(input: f32) -> u32 {
    sensory_encoders::linear_encoder(input, 1000.)
}

fn main() {
    let sensor_names = ["1", "2", "3", "4"];

    let mut sensors: Vec<Box<dyn Sensor>> = Vec::new();

    for name in &sensor_names {
        sensors.push(Box::new(ConstantSensor::new(0.5, name.parse().unwrap())));
    }

    let actuator_names = ["yote", "yang", "yoder"];

    let mut actuators: Vec<Box<dyn Actuator>> = Vec::new();

    for name in &actuator_names {
        actuators.push(Box::new(BasicActuator::new(name.parse().unwrap())));
    }

    let reflexes = vec![
        Reflex::new(
            "1".parse().unwrap(),
            "yote".parse().unwrap(),
            SynapticType::Excitatory,
            20.,
        ),
        Reflex::new(
            "3".parse().unwrap(),
            "yang".parse().unwrap(),
            SynapticType::Excitatory,
            20.,
        ),
        Reflex::new(
            "1".parse().unwrap(),
            "yoder".parse().unwrap(),
            SynapticType::Excitatory,
            20.,
        ),
        Reflex::new(
            "2".parse().unwrap(),
            "yoder".parse().unwrap(),
            SynapticType::Excitatory,
            20.,
        ),
    ];

    let ecp_g = Box::new(BoxEcp::new(10_u32.pow(3), 4, 3, 216));

    let encephalon = Encephalon::new(
        ecp_g,
        sensors,
        actuators,
        10.,
        2. / 101.,
        Rc::new(|| Box::new(RefCell::new(SigmoidStrength::new(9., 1., 0.1)))),
        0.1,
        64,
        encoder,
        reflexes,
    );

    encephalon.run_n_cycles(3000);
}

struct ConstantSensor {
    value: f32,
    name: String,
}

impl ConstantSensor {
    fn new(value: f32, name: String) -> ConstantSensor {
        ConstantSensor { value, name }
    }
}

impl Sensor for ConstantSensor {
    fn measure(&mut self) -> f32 {
        self.value
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

struct BasicActuator {
    name: String,
    value: RefCell<f32>,
}

impl BasicActuator {
    fn new(name: String) -> BasicActuator {
        BasicActuator {
            name,
            value: RefCell::new(0.0),
        }
    }
}

impl Actuator for BasicActuator {
    fn set_control_value(&self, value: f32) {
        *self.value.borrow_mut() = value;
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
