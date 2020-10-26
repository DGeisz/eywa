use warp::Filter;
use tokio::sync::{watch, mpsc};
use std::cell::RefCell;

use eywa::Sensor;
use eywa::Actuator;

#[tokio::main]
async fn main() {

    let mut sensors = Vec::new();

    let (mut forward_tx, mut forward_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        forward_rx,
        "forward".into(),
    ));

    let (mut forward_pain_tx, mut forward_pain_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        forward_pain_rx,
        "forward_pain".into()
    ));

    let (mut left_tx, mut left_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        left_rx,
        "left".into()
    ));

    let (mut left_pain_tx, mut left_pain_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        left_pain_rx,
        "left_pain".into()
    ));

    let (mut right_tx, mut right_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        right_rx,
        "right".into()
    ));

    let (mut right_pain_tx, mut right_pain_rx) = mpsc::channel::<f32>(10);

    sensors.push(HttpReqSensor::new(
        right_pain_rx,
        "right_pain".into()
    ));


    let hello = warp::path!("hello" / String)
        .map(|name| {
            println!("Got hello req");
            format!("Hello, {}", name)
        });

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

struct SensorSender {
    forward: mpsc::Sender<f32>,
    forward_pain: mpsc::Sender<f32>,
    left: mpsc::Sender<f32>,
    left_pain: mpsc::Sender<f32>,
    right: mpsc::Sender<f32>,
    right_pain: mpsc::Sender<f32>
}

struct ActuatorWatcher {
    left_forward: watch::Receiver<f32>,
    left_backward: watch::Receiver<f32>,
    right_forward: watch::Receiver<f32>,
    right_backward: watch::Receiver<f32>
}

struct HttpReqSensor {
    rx: mpsc::Receiver<f32>,
    name: String,
    cache: RefCell<Option<f32>>
}

impl HttpReqSensor {
    pub fn new(
        rx: mpsc::Receiver<f32>,
        name: String,
    ) -> HttpReqSensor {
        HttpReqSensor {
            rx,
            name,
            cache: RefCell::new(None)
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
    name: String
}

impl HttpResActuator {
    pub fn new(tx: watch::Sender<f32>, name: String) -> HttpResActuator {
        HttpResActuator {
            tx,
            name
        }
    }
}

impl Actuator for HttpResActuator {
    fn set_control_value(&self, value: f32) {
        self.tx.broadcast(value);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
