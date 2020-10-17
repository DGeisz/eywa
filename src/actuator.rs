/// All actuators controlled by the encephalon
/// must implement this trait
pub trait Actuator {
    fn set_value(&self, value: f32);
}
