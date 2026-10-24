//! Contains the base [`App`] structure.
use fent_ecs::{system::IntoSystem, world::World};

use crate::{
    base::{AppState, Exit},
    lifecycle::{ScheduleId, ScheduleLabel, schedule_id},
    plugins::Plugin,
    schedule::Schedules,
};

pub struct App {
    pub(crate) world: World,
    pub(crate) schedules: Schedules,
}

impl Default for App {
    fn default() -> Self {
        let mut world = World::default();

        world.insert_resource(AppState::default());

        Self {
            world,
            schedules: Schedules::default(),
        }
    }
}

impl App {
    pub fn add_schedule<L: ScheduleLabel>(&mut self) -> ScheduleId {
        self.schedules.add_schedule::<L>()
    }

    pub fn set_schedule_condition<L, F>(&mut self, condition: F) -> &mut Self
    where
        L: ScheduleLabel,
        F: Fn(&World) -> bool + 'static,
    {
        self.schedules.set_schedule_condition::<L, F>(condition);

        self
    }

    pub fn add_system<L, S, M>(&mut self, _label: L, sys: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystem<M>,
    {
        self.schedules.add_system(_label, sys);

        self
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn run(&mut self) {
        let order = self.schedules.schedule_order.clone();

        for &id in &order {
            self.schedules.init_schedule(id, &mut self.world);
        }

        let exit = schedule_id::<Exit>();

        'runner: loop {
            for &id in &order {
                if id == exit {
                    continue;
                }

                self.schedules.run_schedule(id, &mut self.world);
            }

            if self
                .world
                .get_resource::<AppState>()
                .unwrap()
                .is_requested()
            {
                self.schedules.run_schedule(exit, &mut self.world);

                break 'runner;
            }
        }
    }
}
