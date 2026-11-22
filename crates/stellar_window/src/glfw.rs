use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use fent_ecs::prelude::*;
use glfw::{Action, Glfw, GlfwReceiver, PWindow, WindowEvent};
use log::{debug, error};
use stellar_input::keyboard::{KeyCode, KeyEvent};

use crate::{NoRenderer, WindowCloseRequested, WindowResized, window::Window};

pub struct GlfwWindowEntry {
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    title: String,
}

#[derive(Default)]
pub struct GlfwWindows {
    pub windows: HashMap<usize, GlfwWindowEntry>,
    pub entity_id: HashMap<usize, Entity>,
    pub next_id: usize,
}

#[derive(Component)]
pub struct GlfwWindowId(pub usize);

pub struct GlfwState {
    pub glfw: Glfw,
    pub windows: GlfwWindows,
}

thread_local! {
    static GLFW: RefCell<Option<GlfwState>> = const { RefCell::new(None) };
}

pub fn with_glfw<R>(f: impl FnOnce(&mut GlfwState) -> R) -> R {
    GLFW.with(|cell| {
        let mut gaurd = cell.borrow_mut();
        let state = gaurd
            .as_mut()
            .expect("Glfw not initialised, or a system is not running on the main thread");
        f(state)
    })
}

pub fn with_window<R>(id: usize, f: impl FnOnce(&mut Glfw, &mut PWindow) -> R) -> Option<R> {
    GLFW.with(|cell| {
        let mut guard = cell.borrow_mut();
        let state = guard.as_mut()?;

        let GlfwState { glfw, windows } = state;
        let entry = windows.windows.get_mut(&id)?;

        Some(f(glfw, &mut entry.window))
    })
}

pub fn required_vulkan_extensions() -> Vec<String> {
    with_glfw(|state| {
        state
            .glfw
            .get_required_instance_extensions()
            .unwrap_or_default()
    })
}

pub fn framebuffer_size(id: usize) -> Option<(u32, u32)> {
    with_window(id, |_, w| {
        let (width, height) = w.get_framebuffer_size();
        (width.max(0) as u32, height.max(0) as u32)
    })
}

pub(crate) fn glfw_init(_marker: NonSendMarker) {
    let glfw = glfw::init(glfw::fail_on_errors).unwrap();

    GLFW.with(|cell| {
        *cell.borrow_mut() = Some(GlfwState {
            glfw,
            windows: GlfwWindows::default(),
        });
    });
}

fn sync_windows(
    glfw: &mut GlfwState,
    mut cmd: Commands,
    alive: Query<&GlfwWindowId>,
    new: Query<(Entity, &Window), Without<GlfwWindowId>>,
) {
    let alive_ids: HashSet<usize> = alive.iter().map(|window_id| window_id.0).collect();

    glfw.windows.windows.retain(|id, _| alive_ids.contains(id));

    glfw.windows
        .entity_id
        .retain(|id, _| alive_ids.contains(id));

    for (entity, config) in new.iter() {
        glfw.glfw
            .window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let Some((mut window, events)) = glfw.glfw.create_window(
            config.inner_size.width(),
            config.inner_size.height(),
            &config.title,
            glfw::WindowMode::Windowed,
        ) else {
            error!("Failed to create GLFW window: \"{}\"", config.title);
            continue;
        };

        window.set_framebuffer_size_polling(true);
        window.set_close_polling(true);
        window.set_key_polling(true);
        window.set_resizable(config.resizable);

        let id = glfw.windows.next_id;
        glfw.windows.next_id += 1;

        glfw.windows.entity_id.insert(id, entity);
        glfw.windows.windows.insert(
            id,
            GlfwWindowEntry {
                window,
                events,
                title: config.title.clone(),
            },
        );

        cmd.insert_component(entity, GlfwWindowId(id));

        if !config.vulkan {
            cmd.insert_component(entity, NoRenderer);
        }

        debug!("Created GLFW window: \"{}\"", config.title);
    }
}

fn pump_events(glfw: &mut GlfwState, mut cmd: Commands, mut key_writer: MessageWriter<KeyEvent>) {
    glfw.glfw.poll_events();

    for (&id, entry) in &glfw.windows.windows {
        let Some(&entity) = glfw.windows.entity_id.get(&id) else {
            continue;
        };

        for (_, event) in glfw::flush_messages(&entry.events) {
            match event {
                WindowEvent::Close => {
                    debug!("Closed GLFW window: \"{}\"", entry.title);

                    cmd.emit(WindowCloseRequested { entity });
                }
                WindowEvent::FramebufferSize(width, height) => {
                    cmd.emit(WindowResized {
                        entity,
                        width: width.max(0) as u32,
                        height: height.max(0) as u32,
                    });
                }
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    let key_code = match_code(key);

                    match action {
                        Action::Press => {
                            key_writer.write(KeyEvent::KeyDown {
                                window: glfw
                                    .windows
                                    .entity_id
                                    .get(&id)
                                    .expect("Failed to retrieve entity by the window's id")
                                    .to_owned(),
                                key_code,
                                repeat: false,
                            });
                        }
                        Action::Release => {
                            key_writer.write(KeyEvent::KeyUp {
                                window: glfw
                                    .windows
                                    .entity_id
                                    .get(&id)
                                    .expect("Failed to retrieve entity by the window's id")
                                    .to_owned(),
                                key_code,
                                repeat: false,
                            });
                        }
                        Action::Repeat => {
                            key_writer.write(KeyEvent::KeyDown {
                                window: glfw
                                    .windows
                                    .entity_id
                                    .get(&id)
                                    .expect("Failed to retrieve entity by the window's id")
                                    .to_owned(),
                                key_code,
                                repeat: true,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn match_code(key: glfw::Key) -> Option<KeyCode> {
    use glfw::Key::*;

    Some(match key {
        A => KeyCode::KeyA,
        B => KeyCode::KeyB,
        C => KeyCode::KeyC,
        D => KeyCode::KeyD,
        E => KeyCode::KeyE,
        F => KeyCode::KeyF,
        G => KeyCode::KeyG,
        H => KeyCode::KeyH,
        I => KeyCode::KeyI,
        J => KeyCode::KeyJ,
        K => KeyCode::KeyK,
        L => KeyCode::KeyL,
        M => KeyCode::KeyM,
        N => KeyCode::KeyN,
        O => KeyCode::KeyO,
        P => KeyCode::KeyP,
        Q => KeyCode::KeyQ,
        R => KeyCode::KeyR,
        S => KeyCode::KeyS,
        T => KeyCode::KeyT,
        U => KeyCode::KeyU,
        V => KeyCode::KeyV,
        W => KeyCode::KeyW,
        X => KeyCode::KeyX,
        Y => KeyCode::KeyY,
        Z => KeyCode::KeyZ,

        Num0 => KeyCode::Digit0,
        Num1 => KeyCode::Digit1,
        Num2 => KeyCode::Digit2,
        Num3 => KeyCode::Digit3,
        Num4 => KeyCode::Digit4,
        Num5 => KeyCode::Digit5,
        Num6 => KeyCode::Digit6,
        Num7 => KeyCode::Digit7,
        Num8 => KeyCode::Digit8,
        Num9 => KeyCode::Digit9,

        F1 => KeyCode::F1,
        F2 => KeyCode::F2,
        F3 => KeyCode::F3,
        F4 => KeyCode::F4,
        F5 => KeyCode::F5,
        F6 => KeyCode::F6,
        F7 => KeyCode::F7,
        F8 => KeyCode::F8,
        F9 => KeyCode::F9,
        F10 => KeyCode::F10,
        F11 => KeyCode::F11,
        F12 => KeyCode::F12,

        Escape => KeyCode::Escape,
        Delete => KeyCode::Delete,
        Home => KeyCode::Home,
        End => KeyCode::End,
        Insert => KeyCode::Insert,
        Backspace => KeyCode::Backspace,
        Tab => KeyCode::Tab,
        Space => KeyCode::Space,
        PageUp => KeyCode::PageUp,
        PageDown => KeyCode::PageDown,
        PrintScreen => KeyCode::PrintScreen,
        Pause => KeyCode::Pause,

        GraveAccent => KeyCode::Backquote,
        Backslash => KeyCode::Backslash,
        LeftBracket => KeyCode::BracketLeft,
        RightBracket => KeyCode::BracketRight,
        Comma => KeyCode::Comma,
        Period => KeyCode::Period,
        Semicolon => KeyCode::Semicolon,
        Equal => KeyCode::Equal,
        Minus => KeyCode::Minus,
        Apostrophe => KeyCode::Quote,

        LeftShift => KeyCode::ShiftLeft,
        RightShift => KeyCode::ShiftRight,
        LeftControl => KeyCode::ControlLeft,
        RightControl => KeyCode::ControlRight,
        LeftAlt => KeyCode::AltLeft,
        RightAlt => KeyCode::AltRight,
        LeftSuper => KeyCode::MetaLeft,
        RightSuper => KeyCode::MetaRight,

        CapsLock => KeyCode::CapsLock,
        NumLock => KeyCode::NumLock,
        ScrollLock => KeyCode::ScrollLock,
        Menu => KeyCode::ContextMenu,

        Kp0 => KeyCode::Numpad0,
        Kp1 => KeyCode::Numpad1,
        Kp2 => KeyCode::Numpad2,
        Kp3 => KeyCode::Numpad3,
        Kp4 => KeyCode::Numpad4,
        Kp5 => KeyCode::Numpad5,
        Kp6 => KeyCode::Numpad6,
        Kp7 => KeyCode::Numpad7,
        Kp8 => KeyCode::Numpad8,
        Kp9 => KeyCode::Numpad9,
        KpAdd => KeyCode::NumpadAdd,
        KpSubtract => KeyCode::NumpadSubtract,
        KpMultiply => KeyCode::NumpadMultiply,
        KpDivide => KeyCode::NumpadDivide,
        KpDecimal => KeyCode::NumpadDecimal,
        KpEnter => KeyCode::NumpadEnter,
        KpEqual => KeyCode::NumpadEqual,

        Right => KeyCode::Right,
        Left => KeyCode::Left,
        Up => KeyCode::Up,
        Down => KeyCode::Down,

        Unknown | Slash | World1 | World2 | Enter | F13 | F14 | F15 | F16 | F17 | F18 | F19
        | F20 | F21 | F22 | F23 | F24 | F25 => KeyCode::Unidentified,
    })
}

pub(crate) fn glfw_frame_begin(
    cmd: Commands,
    cmd2: Commands,
    alive: Query<&GlfwWindowId>,
    new: Query<(Entity, &Window), Without<GlfwWindowId>>,
    key_writer: MessageWriter<KeyEvent>,
    _marker: NonSendMarker,
) {
    with_glfw(|glfw| {
        sync_windows(glfw, cmd, alive, new);
        pump_events(glfw, cmd2, key_writer);
    })
}

pub(crate) fn glfw_update_windows(
    windows: Query<(&Window, &GlfwWindowId)>,
    _marker: NonSendMarker,
) {
    for (window, window_id) in windows.iter() {
        with_glfw(|glfw| {
            let Some(glfw_window) = glfw.windows.windows.get_mut(&window_id.0) else {
                return;
            };

            if glfw_window.title != window.title {
                glfw_window.window.set_title(window.title.as_str());
                glfw_window.title.clone_from(&window.title);
            }
        })
    }
}

pub(crate) fn glfw_drop(_marker: NonSendMarker) {
    let state = GLFW.with(|cell| cell.borrow_mut().take());

    if let Some(state) = state {
        let GlfwState { windows, glfw } = state;

        drop(windows);
        drop(glfw);
    }
}
