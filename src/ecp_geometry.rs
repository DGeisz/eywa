use crate::neuron::RxNeuron;

/// Here ECP stands for "Encephalon".
/// Trait objects of this type correspond to
/// specific geometric configurations of neurons
/// within an encephalon.
pub trait EcpGeometry {
    /// Creates a new EcpGeometry trait object.  All arguments that start
    /// with num are the number of those types of neurons within
    /// this specific geometry.
    ///
    /// This method will create a geometry that is fully complete and has precisely
    /// num_actuator actuator neurons.  It will either round num_plastic_neurons up
    /// or down in order to achieve this goal, so the actual number of plastic neurons
    /// within the may differ from the value passed as a parameter.  If the user
    /// understands how the geometry is created and specifies specific numbers according
    /// to the geometry, then there should be precisely that number of neurons in the
    /// resulting structure
    ///
    /// "Nearby count" is the number of neurons with which FxNeurons
    /// may form synapses. Again this parameter is an upper limit in the same way
    /// the other num_ arguments are
    fn new(desired_num_plastic: u32, num_sensor: u32, num_actuator: u32, nearby_count: u32) -> Self
    where
        Self: Sized;

    /// These methods return the actual number of each type of neuron within
    /// the structure that were created during new
    fn get_num_plastic(&self) -> u32;
    fn get_num_actuator(&self) -> u32;
    fn get_num_sensor(&self) -> u32;

    /// Here "loc" is short for "location," which is represented
    /// by a vector of integers. These methods return the position
    /// hash (and neuron type located at the returned location of
    /// the method for the rx methods) of either the first neuron
    /// specified by this geometry or the next neuron in the geometry
    ///
    /// The next_ methods will return None if they are the last
    /// of this type of neuron within the structure
    fn first_rx_loc(&self) -> (Vec<i32>, String, RxNeuron);
    fn next_rx_loc(&self, curr_loc: Vec<i32>) -> Option<(Vec<i32>, String, RxNeuron)>;
    fn first_sensory_loc(&self) -> (Vec<i32>, String);
    fn next_sensory_loc(&self, curr_loc: Vec<i32>) -> Option<(Vec<i32>, String)>;

    /// Returns the unique hash that corresponds to each location.
    /// This is used by the encephalon to access different neurons
    fn loc_hash(&self, loc: Vec<i32>) -> String;

    /// Returns a random location with the set of locations that
    /// are considered "nearby" loc.  This is crucial to plasticity
    /// and synapse formation
    fn local_random_hash(&self, loc: &Vec<i32>) -> Option<String>;
}
