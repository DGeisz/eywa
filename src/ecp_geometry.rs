use crate::neuron::RxNeuron;
use rand::Rng;

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
    /// may form synapses. This number is rounded down sufficiently to match
    /// present geometry, so look through how each geometry is implemented to figure out
    /// how to accurately specify nearby_count
    fn new(
        desired_num_plastic: u32,
        num_sensory: u32,
        num_actuator: u32,
        nearby_count: u32,
    ) -> Self
    where
        Self: Sized;

    /// These methods return the actual number of each type of neuron within
    /// the structure that were created during new
    fn get_num_plastic(&self) -> u32;
    fn get_num_actuator(&self) -> u32;
    fn get_num_sensory(&self) -> u32;

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
    fn loc_hash(&self, loc: &Vec<i32>) -> String;

    /// Returns a random location with the set of locations that
    /// are considered "nearby" loc.  This is crucial to plasticity
    /// and synapse formation
    fn local_random_hash(&self, loc: &Vec<i32>) -> Option<String>;
}

/// This is the 3D box ecp geometry.  Basically a box of plastic neurons,
/// with actuator neurons embedded into one end of the box, and sensor
/// neurons floating on the outside of the other side of the box
///
/// Note that the Box simply places actuators and sensors in rows on
/// each of their sides for ease of implementation, and because it's
/// not yet clear if spread has any effect on anything
pub struct BoxEcp {
    num_plastic: u32,
    num_actuator: u32,
    num_sensory: u32,
    nearby_side_length: u32,
    side_length: u32,
}

impl EcpGeometry for BoxEcp {
    /// Note that nearby_count is rounded down until it is a perfect cube,
    /// and its cubed root is odd
    fn new(desired_num_plastic: u32, num_sensory: u32, num_actuator: u32, nearby_count: u32) -> Self
    where
        Self: Sized,
    {
        let side_length = (desired_num_plastic as f32).powf(1. / 3.).floor() as u32;

        let area = side_length.pow(2);
        let volume = side_length.pow(3);

        if num_actuator > area {
            panic!(
                "The number of actuators is greater than the neuron area of \
            one side of the box. Either decrease the number of actuators, or increase \
            the size of the box"
            );
        } else if num_sensory > area {
            panic!(
                "The number of sensory neurons is greater than the neuron area of \
            one side of the box. Either decrease the number of sensory neurons, or increase \
            the size of the box"
            );
        }

        let mut nearby_length = (nearby_count as f32).powf(1. / 3.).floor() as u32;

        if nearby_length % 2 == 0 {
            nearby_length -= 1;
        }

        if nearby_length.pow(3) > volume {
            panic!(
                "The number of nearby neurons exceeds the number of neurons in the box. \
            Either decrease the number of nearby neurons, or increase the size the size of \
            the box"
            );
        };

        BoxEcp {
            num_plastic: volume,
            num_actuator,
            num_sensory,
            nearby_side_length: nearby_length,
            side_length,
        }
    }

    fn get_num_plastic(&self) -> u32 {
        self.num_plastic
    }

    fn get_num_actuator(&self) -> u32 {
        self.num_actuator
    }

    fn get_num_sensory(&self) -> u32 {
        self.num_sensory
    }

    fn first_rx_loc(&self) -> (Vec<i32>, String, RxNeuron) {
        let loc = vec![0, 0, 0];

        (loc.clone(), self.loc_hash(&loc), RxNeuron::Plastic)
    }

    fn next_rx_loc(&self, curr_loc: Vec<i32>) -> Option<(Vec<i32>, String, RxNeuron)> {
        if let Some(x) = curr_loc.get(0) {
            if let Some(y) = curr_loc.get(1) {
                if let Some(z) = curr_loc.get(2) {
                    let last_position = (self.side_length - 1) as i32;

                    let new_x;
                    let new_y;
                    let new_z;

                    // Figure out the new position values for each dimension
                    if *x == last_position {
                        if *y == last_position {
                            if *z == last_position {
                                // In this case, curr_loc as the last rx location
                                // within the box, so return None to indicate there
                                // is no other rx location
                                return None;
                            } else {
                                new_x = 0;
                                new_y = 0;
                                new_z = *z + 1;
                            }
                        } else {
                            new_x = 0;
                            new_y = *y + 1;
                            new_z = *z;
                        }
                    } else {
                        new_x = *x + 1;
                        new_y = *y;
                        new_z = *z;
                    }

                    let new_loc = vec![new_x, new_y, new_z];

                    // If new_z is at the final position, then we need to start worrying
                    // about actuator neurons
                    return if new_z == last_position {
                        let plane_position = (new_y * (self.side_length as i32)) + new_x + 1;
                        let is_actuator = plane_position as u32 <= self.num_actuator;

                        if is_actuator {
                            Some((new_loc.clone(), self.loc_hash(&new_loc), RxNeuron::Plastic))
                        } else {
                            Some((new_loc.clone(), self.loc_hash(&new_loc), RxNeuron::Plastic))
                        }
                    } else {
                        Some((new_loc.clone(), self.loc_hash(&new_loc), RxNeuron::Plastic))
                    };
                }
            }
        }
        None
    }

    fn first_sensory_loc(&self) -> (Vec<i32>, String) {
        let loc = vec![0, 0, -1];

        (loc.clone(), self.loc_hash(&loc))
    }

    fn next_sensory_loc(&self, curr_loc: Vec<i32>) -> Option<(Vec<i32>, String)> {
        if let Some(x) = curr_loc.get(0) {
            if let Some(y) = curr_loc.get(1) {
                let last_position = (self.side_length - 1) as i32;

                let new_x;
                let new_y;
                if *x == last_position {
                    if *y == last_position {
                        return None;
                    } else {
                        new_x = 0;
                        new_y = *y + 1;
                    }
                } else {
                    new_x = *x + 1;
                    new_y = *y;
                }

                let new_loc = vec![new_x, new_y];

                return Some((new_loc.clone(), self.loc_hash(&new_loc)));
            }
        }
        None
    }

    fn loc_hash(&self, loc: &Vec<i32>) -> String {
        format!("{:?}", loc)
    }

    fn local_random_hash(&self, loc: &Vec<i32>) -> Option<String> {
        if let Some(x) = loc.get(0) {
            if let Some(y) = loc.get(1) {
                if let Some(z) = loc.get(2) {
                    let last_position = (self.side_length - 1) as i32;

                    let nearby_side_length_i32 = self.nearby_side_length as i32;

                    let dist_from_center = (nearby_side_length_i32 - 1) / 2;

                    let mut bottom_x = x - dist_from_center;
                    let mut bottom_y = y - dist_from_center;
                    let mut bottom_z = z - dist_from_center;

                    if bottom_x < 0 {
                        bottom_x = 0;
                    } else if bottom_x + (nearby_side_length_i32 - 1) > last_position {
                        bottom_x = (last_position - (nearby_side_length_i32 - 1)) as i32
                    }

                    if bottom_y < 0 {
                        bottom_y = 0;
                    } else if bottom_y + (nearby_side_length_i32 - 1) > last_position {
                        bottom_y = (last_position - (nearby_side_length_i32 - 1)) as i32
                    }

                    if bottom_z < 0 {
                        bottom_z = 0;
                    } else if bottom_z + (nearby_side_length_i32 - 1) > last_position {
                        bottom_z = last_position - (nearby_side_length_i32 - 1)
                    }

                    let mut random_gen = rand::thread_rng();

                    let rand_x =
                        random_gen.gen_range(bottom_x, bottom_x + nearby_side_length_i32 - 1);
                    let rand_y =
                        random_gen.gen_range(bottom_y, bottom_y + nearby_side_length_i32 - 1);
                    let rand_z =
                        random_gen.gen_range(bottom_z, bottom_z + nearby_side_length_i32 - 1);

                    return Some(self.loc_hash(&vec![rand_x, rand_y, rand_z]));
                }
            }
        }
        None
    }
}
