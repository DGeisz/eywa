/// All sensors hooking up to an Encephalon must
/// implement this trait
pub trait Sensor {
    fn measure(&self) -> f32;
}
