//! Rendering capabilities for Stellar.
//!
//! Uses Vulkano (Vulkan wrapper) as the graphics library.

use stellar_app::{app::App, base::*, plugins::Plugin};
use stellar_derive::ScheduleLabel;

use crate::renderer::{
    frame_begin, frame_finish, init_renderers, init_swapchains, recreate_swapchains,
    setup_rendering_events,
};

pub mod context;
pub mod renderer;
pub mod vertex;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Startup, setup_rendering_events)
            .add_system(First, init_renderers)
            .add_system(First, init_swapchains)
            .add_system(First, recreate_swapchains)
            .add_system(RenderBegin, frame_begin)
            .add_system(RenderEnd, frame_finish);
    }
}

#[derive(ScheduleLabel)]
#[schedule(priority = 3000)]
pub struct RenderBegin;

#[derive(ScheduleLabel)]
#[schedule(priority = 2000)]
pub struct Render;

#[derive(ScheduleLabel)]
#[schedule(priority = 1000)]
pub struct RenderEnd;
