//! Window module for the Stellar engine

use fent_derive::{Component, Event};
use fent_ecs::entity::Entity;

use stellar_app::{
    app::App,
    base::{Exit, First, Startup},
    plugins::Plugin,
};

use crate::sdl::{sdl_drop, sdl_frame_begin, sdl_init};

pub(crate) mod sdl;

#[derive(Component)]
pub struct Window {
    pub title: String,
    pub inner_size: (u32, u32),
    pub resizable: bool,
}

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

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Startup, sdl_init)
            .add_system(First, sdl_frame_begin)
            .add_system(Exit, sdl_drop);
    }
}
