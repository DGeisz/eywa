use super::encephalon::Encephalon;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub mod synapse;
use crate::neuron::synapse::synaptic_strength::SynapticStrength;
use crate::neuron::synapse::SynapticType;
use synapse::{PlasticSynapse, StaticSynapse, Synapse};

/// All neurons implement the Neuronic trait
pub trait Neuronic {
    fn run_cycle(&self) -> f32;
}

/// Neurons that transmit (hence Tx) impulses to
/// to other neurons implement the TxNeuronic trait
pub trait TxNeuronic {
    /// Fire all neuron synapses
    fn fire_synapses(&self) {
        for p_synapse in self.get_plastic_synapses().iter() {
            p_synapse.fire();
        }

        for s_synapse in self.get_static_synapses().iter() {
            s_synapse.fire();
        }
    }

    /// Add a static synapse with "target" synapse
    /// Typically called at the inception of the encephalon
    fn add_static_synapse(
        &self,
        strength: f32,
        synaptic_type: SynapticType,
        target_neuron: Rc<dyn NeuronicRx>,
    );

    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>>;
    fn get_static_synapses(&self) -> Ref<Vec<StaticSynapse>>;
}

/// Neurons that receive (hence Rx) impulses from
/// other neurons implement the RxNeuronic trait
pub trait RxNeuronic {

    fn intake_synaptic_impulse(&self, impulse: f32);

    /// Returns true if the neuron fired on the
    /// last cycle
    fn fired_on_prev_cycle(&self) -> bool;
}

/// Enum of the different RxNeurons
pub enum RxNeuron {
    Actuator,
    Plastic,
}

/// Trait used for to reference the fact that a neuron
/// implements both RxNeuronic and Neuronic
pub trait NeuronicRx: RxNeuronic + Neuronic {}

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
    fn form_plastic_synapse(&self);

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
    fn new() -> InternalCharge {
        InternalCharge(0.0, 0.0)
    }

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
    /// Gets the next cycle type
    fn next_cycle(&self) -> ChargeCycle {
        match self {
            ChargeCycle::Even => ChargeCycle::Odd,
            ChargeCycle::Odd => ChargeCycle::Even,
        }
    }
}

/// Tracks if neurons fired at particular cycles
struct FireTracker {
    values: (bool, bool),
    last_recorded_current_cycle: ChargeCycle,
    prev_prev: bool,
}

impl FireTracker {
    fn new() -> FireTracker {
        FireTracker {
            values: (false, false),
            last_recorded_current_cycle: ChargeCycle::Even,
            prev_prev: false,
        }
    }

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
pub struct SensoryNeuron {
    encephalon: Rc<Encephalon>,
    period: RefCell<u32>, //This is the period at which the neuron fires
    max_plastic_synapses: usize,
    plastic_synapses: RefCell<Vec<PlasticSynapse>>,
    static_synapses: RefCell<Vec<StaticSynapse>>,
    fire_tracker: RefCell<FireTracker>,
    synaptic_strength_generator: Rc<dyn Fn() -> Box<RefCell<dyn SynapticStrength>>>,
    synapse_type_threshold: f32,
    ema: RefCell<f32>, //Exponential moving average, ie T(n+1) = αI + (1 - α)T(n)
    alpha: f32,        //The constant of the exponential moving average
    loc: Vec<i32>,
}

impl SensoryNeuron {
    pub fn new(
        encephalon: Rc<Encephalon>,
        max_plastic_synapses: usize,
        synaptic_strength_generator: Rc<dyn Fn() -> Box<RefCell<dyn SynapticStrength>>>,
        synapse_type_threshold: f32,
        alpha: f32, //The constant of the exponential moving average
        loc: Vec<i32>,
    ) -> SensoryNeuron {
        SensoryNeuron {
            encephalon,
            period: RefCell::new(0),
            max_plastic_synapses,
            plastic_synapses: RefCell::new(Vec::new()),
            static_synapses: RefCell::new(Vec::new()),
            fire_tracker: RefCell::new(FireTracker::new()),
            synaptic_strength_generator,
            synapse_type_threshold,
            ema: RefCell::new(0.0),
            alpha,
            loc,
        }
    }

    /// Sets the period of this neuron, which
    /// indicates on which cycle values this neuron
    /// should fire
    pub fn set_period(&self, period: u32) {
        *self.period.borrow_mut() = period;
    }
}

impl Neuronic for SensoryNeuron {
    fn run_cycle(&self) -> f32 {
        self.prune_synapses();
        self.form_plastic_synapse();

        let mut fire_tracker = self.fire_tracker.borrow_mut();
        let current_cycle = self.encephalon.get_charge_cycle();

        let mut ema = self.ema.borrow_mut();

        let period = self.period.borrow();

        if *period != 0 && self.encephalon.get_cycle_count() % *period == 0 {
            self.fire_synapses();
            *ema = self.alpha + ((1.0 - self.alpha) * (*ema));
            fire_tracker.set_tracker(current_cycle, true);
        } else {
            *ema = (1.0 - self.alpha) * (*ema);
            fire_tracker.set_tracker(current_cycle, false);
        }

        ema.clone()
    }
}

impl TxNeuronic for SensoryNeuron {
    fn add_static_synapse(
        &self,
        strength: f32,
        synaptic_type: SynapticType,
        target_neuron: Rc<dyn NeuronicRx>,
    ) {
        self.static_synapses.borrow_mut().push(StaticSynapse::new(
            strength,
            synaptic_type,
            target_neuron,
        ));
    }

    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>> {
        self.plastic_synapses.borrow()
    }

    fn get_static_synapses(&self) -> Ref<Vec<StaticSynapse>> {
        self.static_synapses.borrow()
    }
}

impl FxNeuronic for SensoryNeuron {
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
            synapse.connected()
        })
    }

    fn form_plastic_synapse(&self) {
        let mut plastic_synapses = self.plastic_synapses.borrow_mut();
        if plastic_synapses.len() < self.max_plastic_synapses {
            let new_target_neuron = self.encephalon.local_random_neuron(&self.loc);

            let synapse_type = match *self.ema.borrow() < self.synapse_type_threshold {
                true => SynapticType::Excitatory,
                false => SynapticType::Inhibitory,
            };

            if let Some(neuron_ref) = new_target_neuron {
                let new_synapse = PlasticSynapse::new(
                    (self.synaptic_strength_generator)(),
                    synapse_type,
                    neuron_ref,
                );

                plastic_synapses.push(new_synapse);
            }
        }
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
pub struct ActuatorNeuron {
    encephalon: Rc<Encephalon>,
    fire_tracker: RefCell<FireTracker>,
    internal_charge: RefCell<InternalCharge>,
    fire_threshold: f32,
    ema: RefCell<f32>, //Exponential moving average, ie T(n+1) = αI + (1 - α)T(n)
    alpha: f32,        //The constant of the exponential moving average
}

impl ActuatorNeuron {
    pub fn new(
        encephalon: Rc<Encephalon>,
        fire_threshold: f32,
        alpha: f32, //The constant of the exponential moving average
    ) -> ActuatorNeuron {
        ActuatorNeuron {
            encephalon,
            fire_tracker: RefCell::new(FireTracker::new()),
            internal_charge: RefCell::new(InternalCharge::new()),
            fire_threshold,
            ema: RefCell::new(0.0),
            alpha,
        }
    }

    /// Reads this actuator neuron's EMA firing frequency
    pub fn read_ema_frequency(&self) -> f32 {
        self.ema.borrow().clone()
    }
}

impl Neuronic for ActuatorNeuron {
    fn run_cycle(&self) -> f32 {
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

        ema.clone()
    }
}

impl RxNeuronic for ActuatorNeuron {
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

impl NeuronicRx for ActuatorNeuron {}

/// This is your standard neuron present in the
/// encephalon.  Basically everything about this
/// neuron isn't fixed.  It's incoming or outgoing
/// synapses are subject to change based on its
/// environment
pub struct PlasticNeuron {
    encephalon: Rc<Encephalon>,
    internal_charge: RefCell<InternalCharge>,
    fire_threshold: f32,
    fire_tracker: RefCell<FireTracker>,
    max_plastic_synapses: usize,
    plastic_synapses: RefCell<Vec<PlasticSynapse>>,
    static_synapses: RefCell<Vec<StaticSynapse>>,
    synaptic_strength_generator: Rc<dyn Fn() -> Box<RefCell<dyn SynapticStrength>>>,
    synapse_type_threshold: f32,
    ema: RefCell<f32>, //Exponential moving average, ie T(n+1) = αI + (1 - α)T(n)
    alpha: f32,        //The constant of the exponential moving average
    loc: Vec<i32>,
}

impl PlasticNeuron {
    pub fn new(
        encephalon: Rc<Encephalon>,
        fire_threshold: f32,
        max_plastic_synapses: usize,
        synaptic_strength_generator: Rc<dyn Fn() -> Box<RefCell<dyn SynapticStrength>>>,
        synapse_type_threshold: f32,
        alpha: f32, //The constant of the exponential moving average
        loc: Vec<i32>,
    ) -> PlasticNeuron {
        PlasticNeuron {
            encephalon,
            fire_threshold,
            internal_charge: RefCell::new(InternalCharge::new()),
            fire_tracker: RefCell::new(FireTracker::new()),
            max_plastic_synapses,
            plastic_synapses: RefCell::new(Vec::new()),
            static_synapses: RefCell::new(Vec::new()),
            synaptic_strength_generator,
            synapse_type_threshold,
            ema: RefCell::new(0.0),
            alpha,
            loc,
        }
    }
}

impl Neuronic for PlasticNeuron {
    fn run_cycle(&self) -> f32 {
        self.prune_synapses();
        self.form_plastic_synapse();

        let current_cycle = self.encephalon.get_charge_cycle();
        let mut internal_charge = self.internal_charge.borrow_mut();
        let mut fire_tracker = self.fire_tracker.borrow_mut();

        let mut ema = self.ema.borrow_mut();

        if internal_charge.get_charge(current_cycle) > self.fire_threshold {
            self.fire_synapses();
            *ema = self.alpha + ((1.0 - self.alpha) * (*ema));
            fire_tracker.set_tracker(current_cycle, true);
        } else {
            *ema = (1.0 - self.alpha) * (*ema);
            fire_tracker.set_tracker(current_cycle, false);
        }

        // println!("This is current ema: {}, and fire_count: {}", *ema, fire_count);

        internal_charge.reset_charge(current_cycle);

        ema.clone()
    }
}

impl RxNeuronic for PlasticNeuron {
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

impl NeuronicRx for PlasticNeuron {}

impl TxNeuronic for PlasticNeuron {
    fn add_static_synapse(
        &self,
        strength: f32,
        synaptic_type: SynapticType,
        target_neuron: Rc<dyn NeuronicRx>,
    ) {
        self.static_synapses.borrow_mut().push(StaticSynapse::new(
            strength,
            synaptic_type,
            target_neuron,
        ));
    }

    fn get_plastic_synapses(&self) -> Ref<Vec<PlasticSynapse>> {
        self.plastic_synapses.borrow()
    }

    fn get_static_synapses(&self) -> Ref<Vec<StaticSynapse>> {
        self.static_synapses.borrow()
    }
}

impl FxNeuronic for PlasticNeuron {
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
            synapse.connected()
        })
    }

    fn form_plastic_synapse(&self) {
        let mut plastic_synapses = self.plastic_synapses.borrow_mut();

        if plastic_synapses.len() < self.max_plastic_synapses {
            let new_target_neuron = self.encephalon.local_random_neuron(&self.loc);

            let synapse_type = match *self.ema.borrow() < self.synapse_type_threshold {
                true => SynapticType::Excitatory,
                false => SynapticType::Inhibitory,
            };

            if let Some(neuron_ref) = new_target_neuron {
                let new_synapse = PlasticSynapse::new(
                    (self.synaptic_strength_generator)(),
                    synapse_type,
                    neuron_ref,
                );

                plastic_synapses.push(new_synapse);
            }
        }
    }

    fn fired_on_prev_prev(&self) -> bool {
        self.fire_tracker
            .borrow()
            .fired_on_prev_prev(self.encephalon.get_charge_cycle())
    }
}
