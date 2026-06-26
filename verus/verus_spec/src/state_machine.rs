// RustShield — state_machine: Generic device state machine specification

use core::marker::PhantomData;

/// A state machine for tracking device state.
///
/// This is the specification type used by both the Verus proofs
/// (for formal verification) and the eBPF canary (for runtime
/// behavioral tracking).
pub trait DeviceStateMachine {
    type State: Clone + Copy + Eq + core::fmt::Debug;

    fn initial_state() -> Self::State;
    fn is_valid_transition(from: Self::State, to: Self::State) -> bool;
    fn all_states() -> &'static [Self::State];
}

/// Standard NIC driver states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NicState {
    Down,
    Up,
    Suspended,
    Error,
    Reset,
}

impl DeviceStateMachine for NicState {
    type State = NicState;

    fn initial_state() -> Self::State {
        NicState::Down
    }

    fn is_valid_transition(from: NicState, to: NicState) -> bool {
        matches!(
            (from, to),
            (NicState::Down, NicState::Up)
                | (NicState::Up, NicState::Down)
                | (NicState::Up, NicState::Suspended)
                | (NicState::Suspended, NicState::Up)
                | (NicState::Up, NicState::Error)
                | (NicState::Error, NicState::Reset)
                | (NicState::Reset, NicState::Down)
                | (NicState::Suspended, NicState::Down)
        )
    }

    fn all_states() -> &'static [NicState] {
        &[NicState::Down, NicState::Up, NicState::Suspended, NicState::Error, NicState::Reset]
    }
}

/// Standard block driver states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockState {
    Down,
    Up,
    Suspended,
    Error,
}

impl DeviceStateMachine for BlockState {
    type State = BlockState;

    fn initial_state() -> Self::State {
        BlockState::Down
    }

    fn is_valid_transition(from: BlockState, to: BlockState) -> bool {
        matches!(
            (from, to),
            (BlockState::Down, BlockState::Up)
                | (BlockState::Up, BlockState::Down)
                | (BlockState::Up, BlockState::Suspended)
                | (BlockState::Suspended, BlockState::Up)
                | (BlockState::Up, BlockState::Error)
                | (BlockState::Error, BlockState::Down)
        )
    }

    fn all_states() -> &'static [BlockState] {
        &[BlockState::Down, BlockState::Up, BlockState::Suspended, BlockState::Error]
    }
}

/// A tracked state machine instance.
pub struct TrackedStateMachine<S: DeviceStateMachine> {
    current: S::State,
    _marker: PhantomData<S>,
}

impl<S: DeviceStateMachine> TrackedStateMachine<S> {
    pub fn new() -> Self {
        Self {
            current: S::initial_state(),
            _marker: PhantomData,
        }
    }

    pub fn transition(&mut self, target: S::State) -> Result<(), &'static str> {
        if S::is_valid_transition(self.current, target) {
            self.current = target;
            Ok(())
        } else {
            Err("Invalid state transition")
        }
    }

    pub fn current(&self) -> S::State {
        self.current
    }
}
