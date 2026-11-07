//! Window module for the Stellar engine

use fent_derive::{Component, Event, Resource};
use fent_ecs::{
    entity::Entity,
    system::{
        chain_system::IntoSystemTuple,
        impl_param::{Res, ResMut, command::Commands},
    },
};

use stellar_app::{
    app::App,
    base::{AppState, Exit, First, Startup},
    plugins::Plugin,
};

use crate::{
    sdl::{sdl_drop, sdl_frame_begin, sdl_init},
    window::Window,
};

pub(crate) mod sdl;
pub mod window;

#[derive(Component)]
pub struct PrimaryWindow;

#[derive(Event)]
pub struct WindowCloseRequested {
    pub entity: Entity,
}

#[derive(Event)]
pub struct WindowResized {
    pub entity: Entity,
    pub width: u32,
    pub height: u32,
}

pub struct WindowPlugin {
    /// If a window is to be created on startup,
    /// Useful if you want to run code pre-window creation.
    pub startup_window: bool,
    /// A window to add when starting,
    /// If [`None`], a window is created as default (with accompanied [`PrimaryWindow`] flag.
    /// Else it is added to an [`Entity`] with the flag aswell.
    pub window: Option<Window>,
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowCreateData {
            startup_window: self.startup_window,
            window: self.window.clone(),
        })
        .add_system(Startup, (sdl_init, create_window).chain())
        .add_system(First, sdl_frame_begin)
        .add_system(Exit, sdl_drop);
    }
}

#[derive(Resource)]
struct WindowCreateData {
    startup_window: bool,
    window: Option<Window>,
}

fn create_window(mut cmd: Commands, create_data: Res<WindowCreateData>) {
    if !create_data.startup_window {
        return;
    }

    cmd.spawn((
        create_data.window.clone().unwrap_or_default(),
        PrimaryWindow,
    ));

    cmd.on(
        |_event: &WindowCloseRequested, mut state: ResMut<AppState>| {
            state.request();
        },
    );

    cmd.remove_resource::<WindowCreateData>();
}
