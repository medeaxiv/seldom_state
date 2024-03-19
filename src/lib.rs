//! Component-based state machine plugin for Bevy. Useful for AI, player state, and other entities
//! that occupy different states.

#![warn(missing_docs)]

mod machine;
pub mod set;
mod state;
pub mod trigger;

use std::marker::PhantomData;

use bevy::{ecs::schedule::ScheduleLabel, utils::intern::Interned};
use machine::machine_plugin;
use prelude::*;
use trigger::trigger_plugin;

/// Add to your app to use this crate
#[derive(Debug)]
pub struct StateMachinePlugin<T>
where
    T: Send + Sync + 'static,
{
    schedule: Interned<dyn ScheduleLabel>,
    phantom: PhantomData<T>,
}

impl<T> StateMachinePlugin<T>
where
    T: Send + Sync + 'static,
{
    /// Creates an instance of StateMachinePlugin running its machines in the specified schedule
    pub fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
            phantom: PhantomData,
        }
    }
}

impl<T> Default for StateMachinePlugin<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(PostUpdate)
    }
}

impl<T> Plugin for StateMachinePlugin<T>
where
    T: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.fn_plugin(state_machine_plugin::<T>(self.schedule));
    }
}

/// Function called by [`StateMachinePlugin`]. You may instead call it directly or use
/// `seldom_fn_plugin`, which is another crate I maintain.
pub fn state_machine_plugin<T>(schedule: impl ScheduleLabel + Clone) -> impl FnOnce(&mut App)
where
    T: Send + Sync + 'static,
{
    move |app| {
        app.fn_plugin(machine_plugin::<T>(schedule.clone()))
            .fn_plugin(trigger_plugin::<T>(schedule.clone()));
    }
}

/// Module for convenient imports. Use with `use seldom_state::prelude::*;`.
pub mod prelude {
    pub(crate) use bevy::prelude::*;
    #[cfg(feature = "leafwing_input")]
    pub(crate) use leafwing_input_manager::prelude::*;
    pub(crate) use seldom_fn_plugin::FnPluginExt;

    #[cfg(feature = "leafwing_input")]
    pub use crate::trigger::{
        action_data, axis_pair, axis_pair_length_bounds, axis_pair_max_length,
        axis_pair_min_length, axis_pair_rotation_bounds, axis_pair_unbounded, clamped_axis_pair,
        clamped_axis_pair_length_bounds, clamped_axis_pair_max_length,
        clamped_axis_pair_min_length, clamped_axis_pair_rotation_bounds,
        clamped_axis_pair_unbounded, clamped_value, clamped_value_max, clamped_value_min,
        clamped_value_unbounded, just_pressed, just_released, pressed, value, value_max, value_min,
        value_unbounded,
    };
    pub use crate::{
        machine::StateMachine,
        state::{AnyState, EntityState},
        state_machine_plugin,
        trigger::{always, done, on_event, Done, IntoTrigger, Never, Trigger},
        StateMachinePlugin,
    };
}
