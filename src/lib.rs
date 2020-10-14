mod encephalon;
mod neuron;
mod neuron_interfaces;
mod actuator;
mod sensor;

pub mod eywa {

    use uuid::Uuid;

    pub fn get_uuid() -> Uuid {
        Uuid::new_v4()
    }
}
