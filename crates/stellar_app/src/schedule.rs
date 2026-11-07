use fent_ecs::{schedule::Schedule, system::IntoSystem, world::World};

use crate::lifecycle::{ScheduleId, ScheduleLabel, schedule_id};

#[allow(clippy::type_complexity)]
pub(crate) struct ScheduleEntry {
    priority: isize,
    schedule: Schedule,
    // How the fuck is this considered complex?
    condition: Option<Box<dyn Fn(&World) -> bool>>,
}

#[derive(Default)]
pub(crate) struct Schedules {
    pub(crate) schedules: Vec<Option<ScheduleEntry>>,
    pub(crate) schedule_order: Vec<ScheduleId>,
}

impl Schedules {
    pub(crate) fn add_schedule<L: ScheduleLabel>(&mut self) -> ScheduleId {
        let id = schedule_id::<L>();

        if self.schedules.len() <= id {
            self.schedules.resize_with(id + 1, || None);
        }

        if self.schedules[id].is_none() {
            self.schedules[id] = Some(ScheduleEntry {
                priority: L::priority(),
                schedule: Schedule::default(),
                condition: None,
            });

            self.order_schedules();
        }

        id
    }

    fn order_schedules(&mut self) {
        self.schedule_order = self
            .schedules
            .iter()
            .enumerate()
            .filter_map(|(id, entry)| entry.as_ref().map(|_| id))
            .collect();

        self.schedule_order
            .sort_by_key(|&id| std::cmp::Reverse(self.schedules[id].as_ref().unwrap().priority));
    }

    pub fn set_schedule_condition<L, F>(&mut self, condition: F)
    where
        L: ScheduleLabel,
        F: Fn(&World) -> bool + 'static,
    {
        let id = self.add_schedule::<L>();

        self.schedules[id].as_mut().unwrap().condition = Some(Box::new(condition));
    }

    pub fn add_system<L, S, M>(&mut self, _label: L, sys: S)
    where
        L: ScheduleLabel,
        S: IntoSystem<M>,
    {
        let id = self.add_schedule::<L>();

        self.schedules[id]
            .as_mut()
            .unwrap()
            .schedule
            .insert_system(sys);
    }

    pub(crate) fn run_schedule(&mut self, id: ScheduleId, world: &mut World) {
        let should_run = {
            let Some(Some(entry)) = self.schedules.get(id) else {
                return;
            };

            match &entry.condition {
                Some(condition) => condition(world),
                None => true,
            }
        };

        if !should_run {
            return;
        }

        if let Some(Some(entry)) = self.schedules.get_mut(id) {
            entry.schedule.run(world);
        }
    }

    pub(crate) fn init_schedule(&mut self, id: ScheduleId, world: &mut World) {
        if let Some(Some(entry)) = self.schedules.get_mut(id) {
            entry.schedule.initialize(world).unwrap();
        }
    }
}
