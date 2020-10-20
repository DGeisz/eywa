use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;
use synaptic_strength::SynapticStrength;

use crate::neuron::NeuronicRx;

/// All synapses have the capability to fire
pub trait Synapse {
    /// Fires the synapse. Pretty basic
    fn fire(&self);
}

/// A synapse can strengthen and weaken in different
/// ways, and the synaptic_strength module provides a toolbox
/// of different methods or curves used for synaptic strength
pub mod synaptic_strength {
    pub trait SynapticStrength {
        /// Simply return the strength of the synapse
        fn get_strength(&self) -> f32;
        /// Strengthen the synapse by one increment
        fn strengthen(&mut self);
        /// Weaken the synapse by one increment
        fn weaken(&mut self);
        /// Returns whether the synaptic strength is
        /// above the weakness threshold
        fn above_weakness_threshold(&self) -> bool;
    }

    /// This synaptic strength follows a sigmoid curve,
    /// so strengthen moves the x_value to the right by
    /// a fixed margin, and weaken moves the x_value to the
    /// left by that same margin
    pub struct SigmoidStrength {
        x_value: f32,
        x_incr: f32,
        max_value: f32,
        weakness_threshold: f32,
    }

    impl SigmoidStrength {
        /// Returns a new sigmoid strength starting with an x_value of 0
        pub fn new(max_value: f32, weakness_threshold: f32, x_incr: f32) -> SigmoidStrength {
            SigmoidStrength {
                x_value: 0.0,
                x_incr,
                max_value,
                weakness_threshold,
            }
        }

        /// Returns a new sigmoid strength starting at a custom x_value
        pub fn new_custom_x(
            max_value: f32,
            weakness_threshold: f32,
            x_incr: f32,
            x_value: f32,
        ) -> SigmoidStrength {
            SigmoidStrength {
                x_incr,
                x_value,
                max_value,
                weakness_threshold,
            }
        }
    }

    impl SynapticStrength for SigmoidStrength {
        fn get_strength(&self) -> f32 {
            self.max_value / (1.0 + (-1. * self.x_value).exp())
        }

        fn strengthen(&mut self) {
            self.x_value += self.x_incr;
        }

        fn weaken(&mut self) {
            self.x_value -= self.x_incr;
        }

        fn above_weakness_threshold(&self) -> bool {
            self.get_strength() > self.weakness_threshold
        }
    }

    /// This type of strength strengthens or weakens
    /// the strength of the synapse by an amount proportional
    /// to the different between self.strength and zero, or
    /// self.strength and self.max_value
    ///
    /// I'm calling it Em (Exponential moving) strength because
    /// it's somewhat similar in implementation to an Exponential
    /// Moving Average
    pub struct EmStrength {
        strength: f32,
        max_value: f32,
        weakness_threshold: f32,
        alpha: f32, //Constant that governs growth and decay
    }

    impl EmStrength {
        /// Makes an EmStrength who's strength starts at half
        /// it's max_value
        pub fn new(max_value: f32, weakness_threshold: f32, alpha: f32) -> EmStrength {
            EmStrength {
                strength: max_value / 2.,
                max_value,
                weakness_threshold,
                alpha,
            }
        }

        /// Makes an EmStrength with a specific starting strength
        pub fn new_custom(
            strength: f32,
            max_value: f32,
            weakness_threshold: f32,
            alpha: f32,
        ) -> EmStrength {
            EmStrength {
                strength,
                max_value,
                weakness_threshold,
                alpha,
            }
        }
    }

    impl SynapticStrength for EmStrength {
        fn get_strength(&self) -> f32 {
            self.strength
        }

        fn strengthen(&mut self) {
            self.strength += self.alpha * (self.max_value - self.strength);
        }

        fn weaken(&mut self) {
            self.strength -= self.alpha * self.strength;
        }

        fn above_weakness_threshold(&self) -> bool {
            self.strength > self.weakness_threshold
        }
    }
}

/// Excitatory synapses increase their target
/// neuron's internal charge, inhibitory synapses
/// decrease their target neuron's internal charge
/// to prevent the neuron from firing
#[derive(Copy, Clone)]
pub enum SynapticType {
    Excitatory,
    Inhibitory,
}

impl SynapticType {
    /// Returns the integer modifier which is multiplied
    /// by the synapse strength to produce the impulse passed
    /// to the target neuron during synapse firing
    fn get_synapse_modifier(&self) -> i8 {
        match self {
            Self::Excitatory => 1,
            Self::Inhibitory => -1,
        }
    }
}

/// This is a synapse that changes in strength
/// over time depending on the extent to which
/// it's firing it correlated with it's targets
/// firing. Can be excitatory or inhibitory.  If
/// this synapse strength passes beneath its
/// weakness threshold, it dissolves
pub struct PlasticSynapse {
    strength: Box<RefCell<dyn SynapticStrength>>,
    synaptic_type: SynapticType,
    pub target: Rc<dyn NeuronicRx>,
}

impl PlasticSynapse {
    pub fn new(
        strength: Box<RefCell<dyn SynapticStrength>>,
        synaptic_type: SynapticType,
        target: Rc<dyn NeuronicRx>,
    ) -> PlasticSynapse {
        PlasticSynapse {
            strength,
            synaptic_type,
            target,
        }
    }

    /// Strengthens the connection of the synapse, which
    /// means it both lasts longer, and imparts a greater
    /// impulse on its target whilst firing
    pub fn strengthen(&self) {
        self.strength.borrow_mut().strengthen();
    }

    /// Weakens the connection of the synapse, which means
    /// it decreases its lifetime and imparts a smaller
    /// impulse on its target whilst firing
    pub fn decay(&self) {
        self.strength.borrow_mut().weaken();
    }

    /// Returns whether the synapse is still connected,
    /// in other words, if it's strength is above the weakness
    /// threshold
    pub fn connected(&self) -> bool {
        self.strength.borrow().above_weakness_threshold()
    }
}

impl Synapse for PlasticSynapse {
    fn fire(&self) {
        let impulse = self.strength.borrow().get_strength()
            * (self.synaptic_type.get_synapse_modifier() as f32);

        self.target.intake_synaptic_impulse(impulse);
    }
}

/// This is a synapse that remains fixed
/// throughout time.  It has a constant
/// strength and a constant target
pub struct StaticSynapse {
    strength: f32,
    synaptic_type: SynapticType,
    target: Rc<dyn NeuronicRx>,
}

impl StaticSynapse {
    pub fn new(
        strength: f32,
        synaptic_type: SynapticType,
        target: Rc<dyn NeuronicRx>,
    ) -> StaticSynapse {
        StaticSynapse {
            strength,
            synaptic_type,
            target,
        }
    }
}

impl Synapse for StaticSynapse {
    fn fire(&self) {
        let impulse = self.strength * (self.synaptic_type.get_synapse_modifier() as f32);
        self.target.intake_synaptic_impulse(impulse);
    }
}
