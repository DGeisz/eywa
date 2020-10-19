/// All actuators controlled by the encephalon
/// must implement this trait
pub trait Actuator {
    /// Set the value of this actuator
    fn set_control_value(&self, value: f32);

    /// Gets the unique name of this actuator
    /// This is used to identify this actuator and
    /// form reflexes upon instantiation of the encephalon
    fn get_name(&self) -> String;
}
