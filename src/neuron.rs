use super::encephalon::Encephalon;
use super::neuron_interfaces::ActuatorInterface;
use std::cell::RefCell;
use std::rc::Rc;

mod synapse;
use synapse::{PlasticSynapse, StaticSynapse, Synapse};

/// All neurons implement the Neuronic trait
pub trait Neuronic {
    fn run_cycle(&self);
}

/// Neurons that transmit (hence Tx) impulses to
/// to other neurons implement the TxNeuronic trait
pub trait TxNeuronic {
    fn fire_synapses(&self);
}

/// Enum of all different neurons that implement
/// the the trait RxNeuronic
pub enum TxNeuron<'a> {
    Sensory(Rc<SensoryNeuron<'a>>),
    Plastic(Rc<PlasticNeuron>),
    Reflex(Rc<ReflexNeuron>),
}

/// Neurons that receive (hence Rx) impulses from
/// other neurons implement the RxNeuronic trait
pub trait RxNeuronic {
    /// Receives an impulse from a synapse to
    /// which it is connected
    fn intake_synaptic_impulse(&self, impulse: f32);

    /// Returns true if the neuron fired on the
    /// last cycle
    fn just_fired(&self) -> bool;
}

/// This represents the internal charge of
/// an RxNeuron.  There are two slots to
/// prevent conflicts that happen inherently
/// in the graphical structure of the encephalon
/// (I.e. neurons fires before it receives all
/// proper impulses, or neuron doesn't fire even
/// though it would have received enough impulse
/// later in this cycle)
pub struct InternalCharge(f32, f32);

impl InternalCharge {
    fn get_charge(&self, cycle: ChargeCycle) -> f32 {
        match cycle {
            ChargeCycle::Even => self.0,
            ChargeCycle::Odd => self.1,
        }
    }

    fn set_charge(&mut self, cycle: ChargeCycle, charge: f32) {
        match cycle {
            ChargeCycle::Even => self.0 = charge,
            ChargeCycle::Odd => self.1 = charge,
        }
    }

    fn incr_next_charge(&mut self, cycle: ChargeCycle, incr_charge: f32) {
        let next_cycle = cycle.next_cycle();
        self.set_charge(next_cycle, self.get_charge(next_cycle) + incr_charge);
    }
}

/// Represents one of two different types of
/// Internal Charge Cycles that occur within
/// a neuron.  Again, this is used to prevent
/// encephalon graphical conflicts
#[derive(Copy, Clone)]
pub enum ChargeCycle {
    Even,
    Odd,
}

impl ChargeCycle {
    fn next_cycle(&self) -> ChargeCycle {
        match self {
            ChargeCycle::Even => ChargeCycle::Odd,
            ChargeCycle::Odd => ChargeCycle::Even,
        }
    }

    /// Don't let the implementation confuse you.
    /// Think of this as a black box.  It just happens
    /// to be the case that previous cycle and next cycle
    /// are the same
    fn prev_cycle(&self) -> ChargeCycle {
        self.next_cycle()
    }

    /// Refers to the cycle before the previous cycle.
    /// Again, don't get lost in the sauce.  It just so
    /// happens that this cycle and prev prev cycle are the
    /// same.  Using this methods will make prune_neurons
    /// much more understandable
    fn prev_prev_cycle(&self) -> ChargeCycle {
        self.clone()
    }
}

/// Here Fx stands for "flex" (don't confuse this with
/// Rx or Tx, it has nothing to do with transmission, I
/// just like the lexical symmetry).  Any neuron that displays
/// some level of plasticity implements FxNeuronic.
///
/// Here plasticity refers to neurons whose synapses strengthen,
/// weaken, dissolve, or appear over time
pub trait FxNeuronic {
    /// Strengthens or decays plastic synapses and dissolves
    /// synapses whose strength has fallen beneath it's
    /// weakness threshold
    fn prune_synapses(&mut self);

    /// Creates new synapse
    fn form_synapse(&mut self);
}

/// Tracks if neurons fired at particular cycles
struct FireTracker {
    values: (bool, bool),
    last_recorded_current_cycle: ChargeCycle,
    prev_prev: bool,
}

impl FireTracker {
    /// Returns true if the neuron fired on the previous cycle
    fn fired_on_prev_cycle(&self, cycle: ChargeCycle) -> bool {
        match cycle.next_cycle() {
            ChargeCycle::Even => self.values.0,
            ChargeCycle::Odd => self.values.1,
        }
    }

    /// Returns true if the neuron fired two cycles ago
    fn fired_on_prev_prev(&self, cycle: ChargeCycle) -> bool {
        if self.last_recorded_current_cycle == cycle {
            self.prev_prev
        } else {
            match cycle {
                ChargeCycle::Even => self.values.0,
                ChargeCycle::Odd => self.values.1,
            }
        }
    }

    /// Sets the tracker for the current cycle
    fn set_tracker(&mut self, cycle: ChargeCycle, fired: bool) {
        self.last_recorded_current_cycle = cycle;
        self.prev_prev = match cycle {
            ChargeCycle::Even => self.values.0,
            ChargeCycle::Odd => self.values.1
        };

        match cycle {
            ChargeCycle::Even => self.values.0 = fired,
            ChargeCycle::Odd => self.values.1 = fired,
        }
    }
}

/// A neuron that sends encoded sensory information into
/// an encephalon
pub struct SensoryNeuron<'a> {
    encephalon: &'a Encephalon,
    period: RefCell<u32>, //This is the period at which the neuron fires
    plastic_synapses: Vec<PlasticSynapse>,
    static_synapses: Vec<StaticSynapse>,
    pub just_fired: bool,
}

impl<'a> SensoryNeuron<'a> {
    pub fn set_period(&self, period: u32) {
        *self.period.borrow_mut() = period;
    }
}

impl Neuronic for SensoryNeuron<'_> {
    fn run_cycle(&self) {
        if self.encephalon.get_cycle_count() % *self.period.borrow() == 0 {
            self.fire_synapses();
        }
    }
}

impl TxNeuronic for SensoryNeuron<'_> {
    fn fire_synapses(&self) {
        for p_synapse in &self.plastic_synapses {
            p_synapse.fire();
        }

        for s_synapse in &self.static_synapses {
            s_synapse.fire();
        }
    }
}

impl FxNeuronic for SensoryNeuron<'_> {
    fn prune_synapses(&mut self) {
        let synapses = &mut self.plastic_synapses;
        let synapses_fired = self.just_fired;

        synapses.retain(|synapse| {
            if synapses_fired {
                if synapse.target.just_fired() {
                    synapse.strengthen();
                } else {
                    synapse.decay();
                }
            }
            let strength = synapse.strength.borrow();
            *strength > synapse.weakness_threshold
        })
    }

    fn form_synapse(&mut self) {
        // TODO: Impl form synapse for neurons last
    }
}

/// A neuron that receives impulses but only
/// sends its average frequency (calculated via EMA)
/// to an ActuatorInterface
pub struct ActuatorNeuron<'a> {
    encephalon: &'a Encephalon,
    pub just_fired: bool,
    internal_charge: RefCell<InternalCharge>,
    interface: &'a ActuatorInterface<'a>,
    ema: RefCell<f32>, //Exponential moving average, ie T(n+1) = αI + (1 - α)T(n)
    alpha: f32,        //The constant of the exponential moving average
}

impl Neuronic for ActuatorNeuron<'_> {
    fn run_cycle(&self) {}
}

impl RxNeuronic for ActuatorNeuron<'_> {
    fn intake_synaptic_impulse(&self, charge: f32) {
        self.internal_charge
            .borrow_mut()
            .incr_next_charge(self.encephalon.get_charge_cycle(), charge);
    }

    fn just_fired(&self) -> bool {
        self.just_fired
    }
}

/// This is your standard neuron present in the
/// encephalon.  Basically everything about this
/// neuron isn't fixed.  It's incoming or outgoing
/// synapses are subject to change based on its
/// environment
pub struct PlasticNeuron {
    pub just_fired: bool,
}

impl RxNeuronic for PlasticNeuron {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        //TODO: Impl this
    }
    fn just_fired(&self) -> bool {
        self.just_fired
    }
}

/// This is a neuron that is essentially fixed in a
/// particular location, typically between a sensor neuron
/// and an actuator neuron
pub struct ReflexNeuron {
    //TODO: Determine if I even need this in the face of
    // Static synapses, and impl if so
    pub just_fired: bool,
}

impl RxNeuronic for ReflexNeuron {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        //TODO: Impl this
    }

    fn just_fired(&self) -> bool {
        self.just_fired
    }
}

//struct Synapse {
//    strength: RefCell<f32>,
//    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
//    max_impulse: f32,
//    growth_parameter: f32, //Must be between 0 and 1
//    decay_parameter: f32, //Must be between 0 and 1
//    synaptic_type: SynapticType,
//    target: Rc<Neuron>
//}
//
//impl Synapse {
//    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
//        Synapse {
//            strength: RefCell::new(1.),
//            weakness_threshold: 0.1,
//            max_impulse: 11.,
//            growth_parameter: 0.1,
//            decay_parameter: 0.1,
//            synaptic_type,
//            target
//        }
//    }
//
//    fn fire(&self) {
//        self.target.intake_synaptic_impulse(
//            *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32))
//    }
//}
//
//enum SynapticType {
//    Excitatory,
//    Inhibitory
//}
//
//impl SynapticType {
//    fn get_synapse_modifier(&self) -> i8 {
//        match self {
//            Self::Excitatory => 1,
//            Self::Inhibitory => -1,
//        }
//    }
//}

//use std::rc::Rc;
//use std::cell::RefCell;
//use rand::{Rng, random};
//
//pub struct Neuron {
//    neuron_type: NeuronType,
//
//    internal_charge: RefCell<f32>,
//    threshold: f32,
//    just_fired: bool,
//
//    max_axon_synapses: usize,
//    axon_synapses: Vec<Synapse>,
//}
//
//pub enum NeuronType {
//    Sensor(f32),
//    Generic
//}
//
//impl Neuron {
//    pub fn new(neuron_type: NeuronType) -> Neuron {
//        Neuron {
//            neuron_type,
//            internal_charge: RefCell::new(0.),
//            threshold: 10.,
//            just_fired: false,
//            max_axon_synapses: 20,
//            axon_synapses: Vec::new()
//        }
//    }
//
//    pub fn run_cycle(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
//        self.prune_synapses();
//
//        let mut internal_charge = self.internal_charge.borrow_mut();
//        if *internal_charge > self.threshold {
//            self.fire_synapses();
//
//            self.just_fired = true;
//            *internal_charge = 0.;
//        } else {
//            self.just_fired = false;
//        }
//    }
//
//    fn create_random_synapse(&mut self, neuron_pool: &Vec<Rc<Neuron>>) {
//        if self.axon_synapses.len() < self.max_axon_synapses {
//            let random_index = rand::thread_rng().gen_range(0, neuron_pool.len());
//
//            if let Some(neuron) = neuron_pool.get(random_index) {
//                self.create_synapse(neuron);
//            }
//        }
//    }
//
//    pub fn create_synapse(&mut self, target_neuron: &Rc<Neuron>) {
//        self.axon_synapses.push(
//            Synapse::new(Rc::clone(target_neuron), SynapticType::Excitatory));
//    }
//
//    fn prune_synapses(&mut self) {
//        let synapses = &mut self.axon_synapses;
//        let synapses_fired = self.just_fired;
//
//        synapses.retain(|synapse: &Synapse| {
//
//            let mut strength = synapse.strength.borrow_mut();
//
//            if synapses_fired {
//                if synapse.target.just_fired {
//                    *strength += (synapse.max_impulse - *strength) * synapse.growth_parameter;
//                } else {
//                    *strength *= synapse.decay_parameter;
//                }
//            }
//
//            *strength > synapse.weakness_threshold
//        })
//    }
//
//    fn fire_synapses(&self) {
//        for synapse in &self.axon_synapses {
//            synapse.fire();
//        }
//    }
//
//    fn intake_synaptic_impulse(&self, charge: f32) {
//        *self.internal_charge.borrow_mut() += charge;
//    }
//}
//
//
//struct Synapse {
//    strength: RefCell<f32>,
//    weakness_threshold: f32, //If self.strength < self.w_t, then synapse dies
//    max_impulse: f32,
//    growth_parameter: f32, //Must be between 0 and 1
//    decay_parameter: f32, //Must be between 0 and 1
//    synaptic_type: SynapticType,
//    target: Rc<Neuron>
//}
//
//impl Synapse {
//    fn new(target: Rc<Neuron>, synaptic_type: SynapticType) -> Synapse {
//        Synapse {
//            strength: RefCell::new(1.),
//            weakness_threshold: 0.1,
//            max_impulse: 11.,
//            growth_parameter: 0.1,
//            decay_parameter: 0.1,
//            synaptic_type,
//            target
//        }
//    }
//
//    fn fire(&self) {
//        self.target.intake_synaptic_impulse(
//            *self.strength.borrow() * (self.synaptic_type.get_synapse_modifier() as f32))
//    }
//}
//
//enum SynapticType {
//    Excitatory,
//    Inhibitory
//}
//
//impl SynapticType {
//    fn get_synapse_modifier(&self) -> i8 {
//        match self {
//            Self::Excitatory => 1,
//            Self::Inhibitory => -1,
//        }
//    }
//}
