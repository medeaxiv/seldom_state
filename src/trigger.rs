use std::marker::PhantomData;

use bevy::{
    ecs::system::{lifetimeless::SQuery, StaticSystemParam, SystemParam},
    reflect::FromReflect,
};

use crate::{prelude::*, stage::StateStage};

/// Plugin that must be added for a trigger to be checked
///
/// # Panics
///
/// Panics with a system param conflict if the given trigger can access [`StateMachine`]
/// as a parameter
#[derive(Debug)]
pub struct TriggerPlugin<T: Trigger>(PhantomData<T>);

impl<T: Trigger> Plugin for TriggerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.fn_plugin(trigger_plugin::<T>);
    }
}

impl<T: Trigger> Default for TriggerPlugin<T> {
    fn default() -> Self {
        Self(default())
    }
}

/// Function called by [`TriggerPlugin`]. You may instead call it directly
/// or use `seldom_fn_plugin`, which is another crate I maintain.
pub fn trigger_plugin<T: Trigger>(app: &mut App) {
    app.add_system_to_stage(StateStage::Trigger, check_trigger::<T>)
        .add_system_to_stage(StateStage::Trigger, check_trigger::<NotTrigger<T>>);
}

pub(crate) fn trigger_plugin_internal(app: &mut App) {
    app.fn_plugin(trigger_plugin::<AlwaysTrigger>)
        .fn_plugin(trigger_plugin::<DoneTrigger>)
        .add_system_to_stage(StateStage::Transition, remove_done_markers);
}

/// Types that implement this may be used in [`StateMachine`]s to transition from one state
/// to another. [`TriggerPlugin`] must be added for each trigger. Since triggers
/// must implement [`FromReflect`] and [`Reflect`], enums may not be triggers. Look at an example
/// for implementing this trait, since it can be tricky without direction.
pub trait Trigger: 'static + FromReflect + Reflect + Send + Sync {
    /// System parameter provided to [`Trigger::trigger()`]. Must not access [`StateMachine`].
    type Param: SystemParam;

    /// Called for every entity that may transition to a state on this trigger. Return `true`
    /// if it should transition, and `false` if it should not.
    // Maybe remove static typing when we get GATS
    fn trigger(&self, entity: Entity, param: &StaticSystemParam<Self::Param>) -> bool;
}

/// Trigger that always transitions
#[derive(Debug, FromReflect, Reflect)]
pub struct AlwaysTrigger;

impl Trigger for AlwaysTrigger {
    type Param = ();

    fn trigger(&self, _: Entity, _: &StaticSystemParam<Self::Param>) -> bool {
        true
    }
}

/// Trigger that negates the contained trigger. It is works for any trigger that is added
/// by [`TriggerPlugin`].
#[derive(Debug, Deref, DerefMut, FromReflect, Reflect)]
pub struct NotTrigger<T: Trigger>(pub T);

impl<T: Trigger> Trigger for NotTrigger<T> {
    type Param = T::Param;

    fn trigger(&self, entity: Entity, param: &StaticSystemParam<Self::Param>) -> bool {
        let NotTrigger(trigger) = self;
        !trigger.trigger(entity, param)
    }
}

/// Marker component that represents that the current state has completed. Removed
/// from every entity each frame after checking triggers. To be used with [`DoneTrigger`].
/// In Bevy 0.9, this will be an enum with `Success` and `Failure` variants.
#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
pub struct Done {
    success: bool,
}

impl Done {
    /// Success variant
    pub fn success() -> Self {
        Self { success: true }
    }

    /// Failure variant
    pub fn failure() -> Self {
        Self { success: false }
    }
}

/// Trigger that transitions if the entity has the [`Done`] component
/// with the associated `success` value. In Bevy 0.9,
/// this will be an enum with `Success` and `Failure` variants.
// TODO variants
#[derive(Debug, FromReflect, Reflect)]
pub struct DoneTrigger {
    success: bool,
}

impl Trigger for DoneTrigger {
    type Param = SQuery<&'static Done>;

    fn trigger(&self, entity: Entity, param: &StaticSystemParam<Self::Param>) -> bool {
        param
            .get(entity)
            .map(|done| self.success == done.success)
            .unwrap_or(false)
    }
}

impl DoneTrigger {
    /// Success variant
    pub fn success() -> Self {
        Self { success: true }
    }

    /// Failure variant
    pub fn failure() -> Self {
        Self { success: false }
    }
}

fn check_trigger<T: Trigger>(
    mut machines: Query<(Entity, &mut StateMachine)>,
    param: StaticSystemParam<T::Param>,
) {
    for (entity, mut machine) in &mut machines {
        for (i, trigger) in machine.get_triggers::<T>().into_iter().enumerate() {
            if trigger.trigger(entity, &param) {
                machine.mark_trigger::<T>(i);
            }
        }
    }
}

fn remove_done_markers(mut commands: Commands, dones: Query<Entity, With<Done>>) {
    for done in &dones {
        commands.entity(done).remove::<Done>();
    }
}
