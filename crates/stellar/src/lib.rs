//! Stellar engine

pub mod prelude {
    pub use stellar_app::{
        app::App,
        base::{BasePlugin, First, Last, Startup, Update},
        plugins::Plugin,
    };
    pub use stellar_derive::ScheduleLabel;
    pub use stellar_input::keyboard::{KeyCode, KeyEvent};
    pub use stellar_log::{LogLevel, LogPlugin};
    pub use stellar_renderer::{RendererPlugin, vertex::Vertex};
    pub use stellar_window::{
        PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResized,
        window::{Window, WindowSize},
    };

    pub use fent_ecs::prelude::*;
    pub use log::*;
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn base_window() {
        App::default()
            .add_plugin(BasePlugin)
            .add_plugin(LogPlugin::trace())
            .add_plugin(WindowPlugin::default())
            .add_plugin(RendererPlugin)
            .run();
    }
}
