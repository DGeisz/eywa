use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, mpsc::error::TrySendError, watch};
use warp::Filter;

use eywa::{
    ecp_geometry::{BoxEcp, EcpGeometry},
    encephalon::{Encephalon, Reflex},
    neuron::synapse::{synaptic_strength::SigmoidStrength, SynapticType},
    neuron_interfaces::sensory_encoders,
    Actuator, Sensor,
};

// Encephalon Parameters
const FIRE_THRESHOLD: f32 = 10.;
const EMA_ALPHA: f32 = 2. / 101.;
const SYNAPTIC_TYPE_THRESHOLD: f32 = 0.1;
const MAX_PLASTIC_SYNAPSES: usize = 64;

const SIGMOID_MAX_VAL: f32 = 9.0;
const WEAKNESS_THRESHOLD: f32 = 1.0;
const X_INCR: f32 = 0.1;

const ENCODER_Y_INTERCEPT: f32 = 100.0;

fn encoder(input: f32) -> u32 {
    sensory_encoders::linear_encoder(input, ENCODER_Y_INTERCEPT)
}

#[tokio::main]
async fn main() {
    // Initialize the sensors
    let (forward_tx, forward_rx) = mpsc::channel::<f32>(10);
    let forward_name: String = "forward".into();

    let (forward_pain_tx, forward_pain_rx) = mpsc::channel::<f32>(10);
    let forward_pain_name: String = "forward_pain".into();

    let (left_tx, left_rx) = mpsc::channel::<f32>(10);
    let left_name: String = "left".into();

    let (left_pain_tx, left_pain_rx) = mpsc::channel::<f32>(10);
    let left_pain_name: String = "left_pain".into();

    let (right_tx, right_rx) = mpsc::channel::<f32>(10);
    let right_name: String = "right".into();

    let (right_pain_tx, right_pain_rx) = mpsc::channel::<f32>(10);
    let right_pain_name: String = "right_pain".into();

    let (back_tx, back_rx) = mpsc::channel::<f32>(10);
    let back_name: String = "back".into();

    let (back_pain_tx, back_pain_rx) = mpsc::channel::<f32>(10);
    let back_pain_name: String = "back_pain".into();

    // Initialize the actuators

    // lf -> Left Forward
    let (lf_tx, lf_rx) = watch::channel::<f32>(0.0);
    let left_forward_name: String = "left_forward".into();

    // lb -> Left Backward
    let (lb_tx, lb_rx) = watch::channel::<f32>(0.0);
    let left_backward_name: String = "left_backward".into();

    // rf -> Right Forward
    let (rf_tx, rf_rx) = watch::channel::<f32>(0.0);
    let right_forward_name: String = "right_forward".into();

    // rb -> Right Backward
    let (rb_tx, rb_rx) = watch::channel::<f32>(0.0);
    let right_backward_name: String = "right_backward".into();

    // Initialize reflexes

    //Make ecp_geometry
    let ecp_geometry = Box::new(BoxEcp::new(125, 8, 4, 27));

    tokio::spawn(async move {
        let sensors = vec![
            Box::new(HttpReqSensor::new(forward_rx, forward_name.clone())) as Box<dyn Sensor>,
            Box::new(HttpReqSensor::new(
                forward_pain_rx,
                forward_pain_name.clone(),
            )),
            Box::new(HttpReqSensor::new(left_rx, left_name.clone())),
            Box::new(HttpReqSensor::new(left_pain_rx, left_pain_name.clone())),
            Box::new(HttpReqSensor::new(right_rx, right_name.clone())),
            Box::new(HttpReqSensor::new(right_pain_rx, right_pain_name.clone())),
            Box::new(HttpReqSensor::new(back_rx, back_name.clone())),
            Box::new(HttpReqSensor::new(back_pain_rx, back_pain_name.clone())),
        ];

        let actuators = vec![
            Box::new(HttpResActuator::new(lf_tx, left_forward_name.clone())) as Box<dyn Actuator>,
            Box::new(HttpResActuator::new(lb_tx, left_backward_name.clone())),
            Box::new(HttpResActuator::new(rf_tx, right_forward_name.clone())),
            Box::new(HttpResActuator::new(rb_tx, right_backward_name.clone())),
        ];

        let reflexes = vec![
            //Forward Pain
            Reflex::new(
                forward_pain_name.clone(),
                left_forward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                forward_pain_name.clone(),
                left_backward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                forward_pain_name.clone(),
                right_forward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                forward_pain_name.clone(),
                right_backward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            //Left Pain
            Reflex::new(
                left_pain_name.clone(),
                left_forward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                left_pain_name.clone(),
                left_backward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                left_pain_name.clone(),
                right_forward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                left_pain_name.clone(),
                right_backward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            //Right Pain
            Reflex::new(
                right_pain_name.clone(),
                left_forward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                right_pain_name.clone(),
                left_backward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                right_pain_name.clone(),
                right_forward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                right_pain_name.clone(),
                right_backward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            //Back Pain
            Reflex::new(
                back_pain_name.clone(),
                left_forward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                back_pain_name.clone(),
                left_backward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
            Reflex::new(
                back_pain_name.clone(),
                right_forward_name.clone(),
                SynapticType::Excitatory,
                20.0,
            ),
            Reflex::new(
                back_pain_name.clone(),
                right_backward_name.clone(),
                SynapticType::Inhibitory,
                20.0,
            ),
        ];

        let encephalon = Encephalon::new(
            ecp_geometry,
            sensors,
            actuators,
            FIRE_THRESHOLD,
            EMA_ALPHA,
            Rc::new(|| {
                Box::new(RefCell::new(SigmoidStrength::new(
                    SIGMOID_MAX_VAL,
                    WEAKNESS_THRESHOLD,
                    X_INCR,
                )))
            }),
            SYNAPTIC_TYPE_THRESHOLD,
            MAX_PLASTIC_SYNAPSES,
            encoder,
            reflexes,
        );

        loop {
            encephalon.run_cycle();
        }
    });

    let sensor_sender = SensorSender {
        forward: forward_tx,
        forward_pain: forward_pain_tx,
        left: left_tx,
        left_pain: left_pain_tx,
        right: right_tx,
        right_pain: right_pain_tx,
        back: back_tx,
        back_pain: back_pain_tx,
    };

    let actuator_watcher = ActuatorWatcher {
        left_forward: lf_rx,
        left_backward: lb_rx,
        right_forward: rf_rx,
        right_backward: rb_rx,
    };

    // Here's the actual warp server.  Sensactio = Sensors Actuators IO
    let sensactio = warp::put()
        .and(warp::path("sensactio"))
        .and(warp::body::json())
        .map(move |sensory_inputs: HttpSensorBody| {
            let mut sender = sensor_sender.clone();
            let watcher = actuator_watcher.clone();

            // Send in latest sensory inputs
            if let Err(e) = sender.send_all(sensory_inputs) {
                println!("Send error: {:?}", e);
            }

            // Respond with current actuator values
            warp::reply::json(&watcher.get_actuator_values())
        });

    warp::serve(sensactio).run(([127, 0, 0, 1], 4200)).await;
}

#[derive(Serialize, Deserialize)]
struct HttpSensorBody {
    forward: f32,
    forward_pain: f32,
    left: f32,
    left_pain: f32,
    right: f32,
    right_pain: f32,
    back: f32,
    back_pain: f32,
}

#[derive(Serialize, Deserialize)]
struct HttpActuatorResponse {
    left_forward: f32,
    left_backward: f32,
    right_forward: f32,
    right_backward: f32,
}

#[derive(Clone)]
struct SensorSender {
    forward: mpsc::Sender<f32>,
    forward_pain: mpsc::Sender<f32>,
    left: mpsc::Sender<f32>,
    left_pain: mpsc::Sender<f32>,
    right: mpsc::Sender<f32>,
    right_pain: mpsc::Sender<f32>,
    back: mpsc::Sender<f32>,
    back_pain: mpsc::Sender<f32>,
}

impl SensorSender {
    pub fn send_all(&mut self, input: HttpSensorBody) -> Result<(), TrySendError<f32>> {
        self.forward.try_send(input.forward)?;
        self.forward_pain.try_send(input.forward_pain)?;
        self.left.try_send(input.left)?;
        self.left_pain.try_send(input.left_pain)?;
        self.right.try_send(input.right)?;
        self.right_pain.try_send(input.right_pain)?;
        self.back_pain.try_send(input.back_pain)?;
        self.back.try_send(input.back)?;
        Ok(())
    }
}

#[derive(Clone)]
struct ActuatorWatcher {
    left_forward: watch::Receiver<f32>,
    left_backward: watch::Receiver<f32>,
    right_forward: watch::Receiver<f32>,
    right_backward: watch::Receiver<f32>,
}

impl ActuatorWatcher {
    pub fn get_actuator_values(&self) -> HttpActuatorResponse {
        HttpActuatorResponse {
            left_forward: *self.left_forward.borrow(),
            left_backward: *self.left_backward.borrow(),
            right_forward: *self.right_forward.borrow(),
            right_backward: *self.right_backward.borrow(),
        }
    }
}

struct HttpReqSensor {
    rx: mpsc::Receiver<f32>,
    name: String,
    cache: RefCell<Option<f32>>,
}

impl HttpReqSensor {
    pub fn new(rx: mpsc::Receiver<f32>, name: String) -> HttpReqSensor {
        HttpReqSensor {
            rx,
            name,
            cache: RefCell::new(None),
        }
    }
}

impl Sensor for HttpReqSensor {
    fn measure(&mut self) -> f32 {
        if let Ok(measurement) = self.rx.try_recv() {
            *self.cache.borrow_mut() = Some(measurement);
            measurement
        } else if let Some(cached_measurement) = *self.cache.borrow() {
            cached_measurement
        } else {
            0.0
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

struct HttpResActuator {
    tx: watch::Sender<f32>,
    name: String,
}

impl HttpResActuator {
    pub fn new(tx: watch::Sender<f32>, name: String) -> HttpResActuator {
        HttpResActuator { tx, name }
    }
}

impl Actuator for HttpResActuator {
    fn set_control_value(&self, value: f32) {
        if let Err(e) = self.tx.broadcast(value) {
            println!("Error sending actuator value: {:?}", e);
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
