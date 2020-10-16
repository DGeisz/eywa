use super::encephalon::Encephalon;
use super::neuron_interfaces::ActuatorInterface;
use std::cell::{Ref, RefCell};

mod synapse;
use synapse::{PlasticSynapse, StaticSynapse, Synapse};

/// All neurons implement the Neuronic trait
pub trait Neuronic {
    fn run_cycle(&self);
}

/// Neurons that transmit (hence Tx) impulses to
/// to other neurons implement the TxNeuronic trait
pub trait TxNeuronic {
    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>>;

    fn get_static_synapses(&self) -> &Vec<StaticSynapse>;

    fn fire_synapses(&self) {
        for p_synapse in self.get_plastic_synapses().iter() {
            p_synapse.fire();
        }

        for s_synapse in self.get_static_synapses() {
            s_synapse.fire();
        }
    }
}

/// Neurons that receive (hence Rx) impulses from
/// other neurons implement the RxNeuronic trait
pub trait RxNeuronic {
    /// Receives an impulse from a synapse to
    /// which it is connected
    fn intake_synaptic_impulse(&self, impulse: f32);

    /// Returns true if the neuron fired on the
    /// last cycle
    fn fired_on_prev_cycle(&self) -> bool;
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
    fn prune_synapses(&self);

    /// Creates new synapse with another (rx) neuron
    /// within this neurons vicinity
    fn form_synapse(&self);

    /// True if neuron fired 2 cycles ago
    fn fired_on_prev_prev(&self) -> bool;
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

    fn reset_charge(&mut self, cycle: ChargeCycle) {
        match cycle {
            ChargeCycle::Even => self.0 = 0.0,
            ChargeCycle::Odd => self.1 = 0.0,
        }
    }

    fn incr_next_charge(&mut self, cycle: ChargeCycle, incr_charge: f32) {
        let next_cycle = cycle.next_cycle();
        let new_charge = self.get_charge(next_cycle) + incr_charge;
        match next_cycle {
            ChargeCycle::Even => self.0 = new_charge,
            ChargeCycle::Odd => self.1 = new_charge,
        }
    }
}

/// Represents one of two different types of
/// Internal Charge Cycles that occur within
/// a neuron.  Again, this is used to prevent
/// encephalon graphical conflicts
#[derive(Copy, Clone, PartialEq)]
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
            ChargeCycle::Odd => self.values.1,
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
    plastic_synapses: RefCell<Vec<PlasticSynapse>>,
    static_synapses: Vec<StaticSynapse>,
    fire_tracker: RefCell<FireTracker>,
}

impl SensoryNeuron<'_> {
    pub fn set_period(&self, period: u32) {
        *self.period.borrow_mut() = period;
    }
}

impl Neuronic for SensoryNeuron<'_> {
    fn run_cycle(&self) {
        let mut fire_tracker = self.fire_tracker.borrow_mut();
        let current_cycle = self.encephalon.get_charge_cycle();

        self.prune_synapses();
        self.form_synapse();

        if self.encephalon.get_cycle_count() % *self.period.borrow() == 0 {
            self.fire_synapses();
            fire_tracker.set_tracker(current_cycle, true);
        } else {
            fire_tracker.set_tracker(current_cycle, false);
        }
    }
}

impl TxNeuronic for SensoryNeuron<'_> {
    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>> {
        self.plastic_synapses.borrow()
    }

    fn get_static_synapses(&self) -> &Vec<StaticSynapse> {
        &self.static_synapses
    }
}

impl FxNeuronic for SensoryNeuron<'_> {
    fn prune_synapses(&self) {
        let synapses_fired = self.fired_on_prev_prev();
        let mut synapses = self.plastic_synapses.borrow_mut();

        synapses.retain(|synapse| {
            if synapses_fired {
                if synapse.target.fired_on_prev_cycle() {
                    synapse.strengthen();
                } else {
                    synapse.decay();
                }
            }
            let strength = synapse.strength.borrow();
            *strength > synapse.weakness_threshold
        })
    }

    fn form_synapse(&self) {
        // TODO: Impl form synapse for neurons last
    }

    fn fired_on_prev_prev(&self) -> bool {
        self.fire_tracker
            .borrow()
            .fired_on_prev_prev(self.encephalon.get_charge_cycle())
    }
}

/// A neuron that receives impulses but only
/// sends its average frequency (calculated via EMA)
/// to an ActuatorInterface
pub struct ActuatorNeuron<'a> {
    encephalon: &'a Encephalon,
    fire_tracker: RefCell<FireTracker>,
    internal_charge: RefCell<InternalCharge>,
    fire_threshold: f32,
    interface: &'a ActuatorInterface<'a>,
    ema: RefCell<f32>, //Exponential moving average, ie T(n+1) = αI + (1 - α)T(n)
    alpha: f32,        //The constant of the exponential moving average
}

impl Neuronic for ActuatorNeuron<'_> {
    fn run_cycle(&self) {
        let current_cycle = self.encephalon.get_charge_cycle();
        let mut internal_charge = self.internal_charge.borrow_mut();
        let mut ema = self.ema.borrow_mut();
        let mut fire_tracker = self.fire_tracker.borrow_mut();

        if internal_charge.get_charge(current_cycle) > self.fire_threshold {
            *ema = self.alpha + ((1.0 - self.alpha) * (*ema));
            fire_tracker.set_tracker(current_cycle, true);
        } else {
            *ema = (1.0 - self.alpha) * (*ema);
            fire_tracker.set_tracker(current_cycle, false);
        }

        internal_charge.reset_charge(current_cycle);

        self.interface.set_output_from_freq(*ema);
    }
}

impl RxNeuronic for ActuatorNeuron<'_> {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        self.internal_charge
            .borrow_mut()
            .incr_next_charge(self.encephalon.get_charge_cycle(), impulse);
    }

    fn fired_on_prev_cycle(&self) -> bool {
        self.fire_tracker
            .borrow()
            .fired_on_prev_cycle(self.encephalon.get_charge_cycle())
    }
}

/// This is your standard neuron present in the
/// encephalon.  Basically everything about this
/// neuron isn't fixed.  It's incoming or outgoing
/// synapses are subject to change based on its
/// environment
pub struct PlasticNeuron<'a> {
    encephalon: &'a Encephalon,
    internal_charge: RefCell<InternalCharge>,
    fire_threshold: f32,
    fire_tracker: RefCell<FireTracker>,
    plastic_synapses: RefCell<Vec<PlasticSynapse>>,
    static_synapses: Vec<StaticSynapse>,
}

impl<'a> Neuronic for PlasticNeuron<'a> {
    fn run_cycle(&self) {
        let current_cycle = self.encephalon.get_charge_cycle();
        let mut internal_charge = self.internal_charge.borrow_mut();
        let mut fire_tracker = self.fire_tracker.borrow_mut();

        self.prune_synapses();
        self.form_synapse();

        if internal_charge.get_charge(current_cycle) > self.fire_threshold {
            self.fire_synapses();
            fire_tracker.set_tracker(current_cycle, true);
        } else {
            fire_tracker.set_tracker(current_cycle, false);
        }

        internal_charge.reset_charge(current_cycle);
    }
}

impl RxNeuronic for PlasticNeuron<'_> {
    fn intake_synaptic_impulse(&self, impulse: f32) {
        self.internal_charge
            .borrow_mut()
            .incr_next_charge(self.encephalon.get_charge_cycle(), impulse);
    }

    fn fired_on_prev_cycle(&self) -> bool {
        self.fire_tracker
            .borrow()
            .fired_on_prev_cycle(self.encephalon.get_charge_cycle())
    }
}

impl TxNeuronic for PlasticNeuron<'_> {
    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>> {
        self.plastic_synapses.borrow()
    }

    fn get_static_synapses(&self) -> &Vec<StaticSynapse> {
        &self.static_synapses
    }
}

impl FxNeuronic for PlasticNeuron<'_> {
    fn prune_synapses(&self) {
        let synapses_fired = self.fired_on_prev_prev();
        let mut synapses = self.plastic_synapses.borrow_mut();

        synapses.retain(|synapse| {
            if synapses_fired {
                if synapse.target.fired_on_prev_cycle() {
                    synapse.strengthen();
                } else {
                    synapse.decay();
                }
            }
            let strength = synapse.strength.borrow();
            *strength > synapse.weakness_threshold
        })
    }

    fn form_synapse(&self) {
        // TODO: Impl form synapse for neurons last
    }

    fn fired_on_prev_prev(&self) -> bool {
        self.fire_tracker
            .borrow()
            .fired_on_prev_prev(self.encephalon.get_charge_cycle())
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
