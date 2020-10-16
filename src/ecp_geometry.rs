/// Here ECP stands for "Encephalon".
/// Trait objects of this type correspond to
/// specific geometric configurations of neurons
/// within an encephalon.
pub trait EcpGeometry {
    /// Creates a new EcpGeometry trait object.  All arguments that start
    /// with num are the number of those types of neurons within
    /// this specific geometry.  Note that this method will create
    /// the largest structure such that the number of neurons doesn't
    /// exceed the values of the num_ parameters.  This means the resulting
    /// structure may have a fewer number of neurons than specified in the
    /// arguments.  "Nearby count" is the number of neurons with which FxNeurons
    /// may form synapses
    fn new(num_plastic: u32, num_sensor: u32, num_actuator: u32, nearby_count: u32) -> Self
    where
        Self: Sized;

    /// Here "loc" is short for "location," which is represented
    /// by a vector of integers. These methods either return the
    /// first location for each type of neuron, or the location
    /// of the neuron that follows a specific location.  The next_
    /// methods will return None if they are the last of this type of
    /// neuron within the structure
    fn first_plastic_loc(&self) -> Vec<i32>;
    fn next_plastic_loc(&self, curr_loc: Vec<i32>) -> Option<Vec<i32>>;
    fn first_sensory_loc(&self) -> Vec<i32>;
    fn next_sensory_loc(&self, curr_loc: Vec<i32>) -> Option<Vec<i32>>;
    fn first_actuator_loc(&self) -> Vec<i32>;
    fn next_actuator_loc(&self, curr_loc: Vec<i32>) -> Option<Vec<i32>>;

    /// Returns the unique hash that corresponds to each location.
    /// This is used by the encephalon to access different neurons
    fn loc_hash(&self, loc: Vec<i32>) -> String;

    /// Returns a random location with the set of locations that
    /// are considered "nearby" loc.  This is crucial to plasticity
    /// and synapse formation
    fn local_random_hash(&self, loc: &Vec<i32>) -> Option<String>;
}
