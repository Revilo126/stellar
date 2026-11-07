//! Includes base functions of a Stellar app,
//! This includes default updating and exit handling.

// Base labels

use fent_derive::Resource;
use fent_ecs::system::impl_param::command::Commands;

use crate::{app::App, lifecycle::ScheduleLabel, plugins::Plugin};

/// Runs at the beginning of every update (normally)
pub struct First;

impl ScheduleLabel for First {
    fn priority() -> isize {
        isize::MIN + 1
    }
}

/// Runs every update
pub struct Update;

impl ScheduleLabel for Update {
    fn priority() -> isize {
        0
    }
}

/// Runs at the end of every update (normally)
pub struct Last;

impl ScheduleLabel for Last {
    fn priority() -> isize {
        isize::MAX - 1
    }
}

/// Runs only once at the start of an app
pub struct Startup;

impl ScheduleLabel for Startup {}

/// Runs at the end of an app
pub struct Exit;

impl ScheduleLabel for Exit {}

// Base resources

/// Allows [`fent_ecs::system::System`]'s to close/exit a running [`App`].
#[derive(Default, Resource)]
pub struct AppState {
    requested: bool,
}

impl AppState {
    pub fn request(&mut self) {
        self.requested = true;
    }

    pub fn is_requested(&self) -> bool {
        self.requested
    }
}

/// Random data I use for schedule conditions
#[derive(Resource)]
struct LabelData;

// Base system

fn startup_add_data(mut cmd: Commands) {
    cmd.insert_resource(LabelData);
}

// Base plugin

/// Adds all the base functionality of a Stellar app.
pub struct BasePlugins;

impl Plugin for BasePlugins {
    fn build(&self, app: &mut App) {
        app.add_schedule::<First>();
        app.add_schedule::<Update>();
        app.add_schedule::<Last>();

        app.add_schedule::<Startup>();
        app.set_schedule_condition::<Startup, _>(|world| {
            world.get_resource::<LabelData>().is_none()
        });

        app.add_schedule::<Exit>();

        app.add_system(Startup, startup_add_data);
    }
}
