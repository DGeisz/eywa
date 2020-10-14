mod encephalon;
mod neuron;
mod neuron_interfaces;

pub mod eywa {

    use uuid::Uuid;

    pub fn get_uuid() -> Uuid {
        Uuid::new_v4()
    }
}
