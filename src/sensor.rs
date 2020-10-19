/// A sensor is any entity that can take
/// a measurement.  The sensor is responsible
/// for scaling this measurement so that is
/// an analog value between 0 and 1
pub trait Sensor {
    /// Returns a value between 0.0 and 1.0
    fn measure(&self) -> f32;

    /// Gets the unique name of this sensor
    /// This is used to identify this sensor and
    /// form reflexes upon instantiation of the encephalon
    fn get_name(&self) -> String;
}
