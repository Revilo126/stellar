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
    glfw::{glfw_drop, glfw_frame_begin, glfw_init, glfw_update_windows},
    window::Window,
};

pub mod glfw;
pub mod window;

#[derive(Component)]
pub struct PrimaryWindow;

#[derive(Component)]
pub struct NoRenderer;

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
    /// If a window is to be created by the [`WindowPlugin`],
    /// This will be the title of said [`window`].
    /// If [`None`] then the default "Stellar Window" is used.
    ///
    /// Does nothing if a window is supplied.
    pub startup_title: Option<String>,
    /// A window to add when starting,
    /// If [`None`], a window is created as default (with accompanied [`PrimaryWindow`] flag.
    /// Else it is added to an [`Entity`] with the flag aswell.
    pub window: Option<Window>,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            startup_window: true,
            startup_title: None,
            window: None,
        }
    }
}

impl WindowPlugin {
    /// Creates a [`WindowPlugin`] that opens a [`Window`] with the supplied title on startup.
    pub fn new(startup_title: impl Into<String>) -> Self {
        Self {
            startup_window: true,
            startup_title: Some(startup_title.into()),
            window: None,
        }
    }

    /// Creates a [`WindowPlugin`] that adds the accepted [`Window`] on startup.
    pub fn with_window(window: Window) -> Self {
        Self {
            startup_window: true,
            startup_title: None,
            window: Some(window),
        }
    }

    /// Creates a [`WindowPlugin`] that doesn't create any [`Window`] on startup.
    pub fn empty() -> Self {
        Self {
            startup_window: false,
            startup_title: None,
            window: None,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowCreateData {
            startup_window: self.startup_window,
            startup_title: self.startup_title.clone(),
            window: self.window.clone(),
        })
        .add_system(Startup, (glfw_init, create_window).chain())
        .add_system(First, glfw_frame_begin)
        .add_system(First, glfw_update_windows)
        .add_system(Exit, glfw_drop);
    }
}

#[derive(Resource)]
struct WindowCreateData {
    startup_window: bool,
    startup_title: Option<String>,
    window: Option<Window>,
}

fn create_window(mut cmd: Commands, create_data: Res<WindowCreateData>) {
    cmd.on(
        |_event: &WindowCloseRequested, mut state: ResMut<AppState>| {
            state.request();
        },
    );

    if !create_data.startup_window {
        return;
    }

    let window = create_data.window.clone().unwrap_or_else(|| {
        let mut window = Window::default();

        if let Some(title) = &create_data.startup_title {
            window.title = title.clone();
        }

        window
    });

    cmd.spawn((window, PrimaryWindow));

    cmd.remove_resource::<WindowCreateData>();
}
