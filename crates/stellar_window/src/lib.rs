use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use fent_derive::{Component, Event, Resource};
use fent_ecs::entity::Entity;
use fent_ecs::system::impl_param::command::Commands;
use fent_ecs::system::impl_param::queries::{Query, Without};
use fent_ecs::system::impl_param::{Res, ResMut};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::event_loop::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{WindowAttributes, WindowId, Window as WinitWindow};
use stellar_app::lifecycle::AppState;

// Allows objects to be accessed only from the thread it's created on
pub struct SingleThreadOnly<T> {
    value: T,
    owner: ThreadId,
}

unsafe impl<T> Send for SingleThreadOnly<T> {}
unsafe impl<T> Sync for SingleThreadOnly<T> {}

impl<T> SingleThreadOnly<T> {
    pub fn new(value: T) -> Self {
        Self { value, owner: thread::current().id() }
    }
    pub fn get_mut(&mut self) -> &mut T {
        assert_eq!(
            thread::current().id(),
            self.owner,
            "Single thread value used from seperate thread"
        );
        &mut self.value
    }
}

#[derive(Resource)]
struct WinitState {
    event_loop: SingleThreadOnly<EventLoop>,
    handler: Handler,
}

impl WinitState {
    pub fn new() -> (Self, WindowRequests) {
        let event_loop = EventLoop::new().unwrap();
        let proxy = event_loop.create_proxy();
        let shared = Arc::new(Shared {
            queue: Mutex::new(VecDeque::new()),
            exit: AtomicBool::new(false),
        });

        let handler = Handler::new(shared.clone());
        let requests = WindowRequests { shared: shared.clone(), proxy };
        (Self { event_loop: SingleThreadOnly::new(event_loop), handler, }, requests)
    }
}

pub struct Handler {
    shared: Arc<Shared>,
    can_create: bool,
    id_to_entity: HashMap<WindowId, Entity>,
    created: Vec<(Entity, Arc<dyn WinitWindow>)>,
    window_events: Vec<(Entity, WindowEvent)>,
    device_events: Vec<DeviceEvent>,
    closed: Vec<Entity>,
}

impl Handler {
    fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
            can_create: false,
            id_to_entity: HashMap::new(),
            created: Vec::new(),
            window_events: Vec::new(),
            device_events: Vec::new(),
            closed: Vec::new(),
        }
    }

    fn drain_requests(&mut self, el: &dyn ActiveEventLoop) {
        if self.shared.exit.load(Ordering::SeqCst) {
            el.exit();
            return;
        }
        if !self.can_create {
            return;
        }
        while let Some(req) = self.shared.queue.lock().unwrap().pop_front() {
            match req {
                Request::Create(entity, attrs) => match el.create_window(attrs) {
                    Ok(win) => {
                        let win: Arc<dyn WinitWindow> = Arc::from(win);
                        self.id_to_entity.insert(win.id(), entity);
                        self.created.push((entity, win));
                    }
                    Err(e) => eprintln!("Failed to create window for {e}"),
                },
            }
        }
    }
}

impl ApplicationHandler for Handler {
    fn can_create_surfaces(&mut self, el: &dyn ActiveEventLoop) {
        self.can_create = true;
        self.drain_requests(el);
    }
    fn proxy_wake_up(&mut self, el: &dyn ActiveEventLoop) { self.drain_requests(el); }
    fn window_event(&mut self, _: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(&entity) = self.id_to_entity.get(&id) else { return };
        if matches!(event, WindowEvent::CloseRequested) {
            self.closed.push(entity);
            self.id_to_entity.remove(&id);
        }
        self.window_events.push((entity, event));
    }

    fn device_event(&mut self, _: &dyn ActiveEventLoop, _: Option<DeviceId>, e: DeviceEvent) {
        self.device_events.push(e);
    }

    fn about_to_wait(&mut self, el: &dyn ActiveEventLoop)  { self.drain_requests(el); }
}

enum Request {
    Create(Entity, WindowAttributes),
}

struct Shared {
    queue: Mutex<VecDeque<Request>>,
    exit: AtomicBool,
}

#[derive(Resource, Clone)]
pub struct WindowRequests {
    shared: Arc<Shared>,
    proxy: EventLoopProxy,
}

impl WindowRequests {
    pub fn create_window(&self, entity: Entity, attrs: WindowAttributes) {
        self.shared.queue.lock().unwrap().push_back(Request::Create(entity, attrs));
        self.proxy.wake_up();
    }

    pub fn exit(&self) {
        self.shared.exit.store(true, Ordering::SeqCst);
        self.proxy.wake_up();
    }
}

#[derive(Component, Clone)]
pub struct Window {
    pub title: String,
    pub size: (u32, u32),
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub visible: bool,
    pub cursor_visible: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Stellar Window".into(),
            size: (1280, 720),
            resizable: true,
            decorations: true,
            transparent: false,
            visible: true,
            cursor_visible: true,
        }
    }
}

impl From<&Window> for WindowAttributes {
    fn from(w: &Window) -> Self {
        WindowAttributes::default()
            .with_title(w.title.clone())
            .with_surface_size(PhysicalSize::new(w.size.0, w.size.1))
            .with_resizable(w.resizable)
            .with_decorations(w.decorations)
            .with_transparent(w.transparent)
            .with_visible(w.visible)
    }
}

#[derive(Component)] pub struct WindowHandle(pub Arc<dyn WinitWindow>);
#[derive(Component)] pub struct PrimaryWindow;
#[derive(Component)] struct WindowRequested;

#[derive(Event)] pub struct WindowCreated { pub window: Entity }
#[derive(Event)] pub struct WindowResized { pub window: Entity, pub size: (u32, u32) }
#[derive(Event)] pub struct WindowCloseRequested { pub window: Entity }

fn request_windows(
    requests: Res<WindowRequests>,
    mut q: Query<(Entity, &Window), (Without<WindowHandle>, Without<WindowRequested>)>,
    mut cmd: Commands,
) {
    for (entity, w) in q.iter(){
        let attrs = WindowAttributes::default().with_title(w.title.clone());
        requests.create_window(entity, attrs);
        cmd.insert_component(entity, WindowRequested);
    }
}

// pump winit, move buffered results into the world
fn winit_pump(mut state: ResMut<WinitState>, mut cmd: Commands) {
    let state = &mut *state;

    let status = state
        .event_loop
        .get_mut()
        .pump_app_events(Some(Duration::ZERO), &mut state.handler);

    for (entity, win) in state.handler.created.drain(..) {
        cmd.spawn(WindowHandle(win));
        cmd.emit(WindowCreated { window: entity });
    }
    for (window, event) in state.handler.window_events.drain(..) {
        match event {
            WindowEvent::CloseRequested => cmd.emit(WindowCloseRequested { window }),
            WindowEvent::SurfaceResized(s) => {
                cmd.emit(WindowResized { window, size: (s.width, s.height) })
            }
            _ => {}
        }
    }
    for _e in state.handler.device_events.drain(..) {
    }
    for entity in state.handler.closed.drain(..) {
        cmd.despawn(entity);
    }
    if matches!(status, PumpStatus::Exit(_)) {
    }
}

fn exit_on_all_closed(mut q: Query<&mut Window>, mut state: ResMut<AppState>) {
    let mut tot = 0;

    for e in q.iter() {
        tot += 1;
    }

    if tot == 0 {
        state.request();
    }
}