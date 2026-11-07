//! Stellar engine

pub mod prelude {
    pub use stellar_app::{
        app::App,
        base::{BasePlugins, First, Last, Startup, Update},
        plugins::Plugin,
    };
    pub use stellar_derive::ScheduleLabel;
    // pub use stellar_input;
    // pub use stellar_window;

    pub use fent_ecs::prelude::*;
}

#[cfg(test)]
mod test {
    use stellar_app::{app::App, base::BasePlugins};
    use stellar_window::WindowPlugin;

    #[test]
    fn name() {
        App::default()
            .add_plugin(BasePlugins)
            .add_plugin(WindowPlugin {
                startup_window: true,
                window: None,
            })
            .run();
    }
}
