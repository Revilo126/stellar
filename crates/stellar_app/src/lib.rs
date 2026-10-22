//! 'stellar_app' is the Stellar crate in which all aspects relating to the App object are located,
//! This includes lifecycle hooks and the running of said applications.
//!

pub mod app;
pub mod lifecycle;
pub mod plugins;

#[cfg(test)]
mod test {
    use crate::{app::App, lifecycle::ScheduleLabel, plugins::Plugin};

    pub struct TestUpdate;
    impl ScheduleLabel for TestUpdate {
        fn priority() -> isize {
            1
        }
    }

    #[test]
    #[should_panic(expected = "Why not")]
    fn app_systems() {
        fn panic_system() {
            panic!("Why not");
        }

        App::default().add_system(TestUpdate, panic_system).run();
    }

    #[test]
    #[should_panic(expected = "Why not")]
    fn app_plugins_insert() {
        fn panic_system() {
            panic!("Why not");
        }

        pub struct TestPlugin;
        impl Plugin for TestPlugin {
            fn build(&self, app: &mut App) {
                app.add_system(TestUpdate, panic_system);
            }
        }

        App::default().add_plugin(TestPlugin).run();
    }
}
