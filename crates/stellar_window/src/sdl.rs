use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use fent_derive::Component;
use fent_ecs::{
    entity::Entity,
    system::impl_param::{
        ResMut,
        command::Commands,
        messages::MessageWriter,
        non_send::NonSendMarker,
        queries::{Query, Without},
    },
};
use log::info;
use sdl3::{
    EventPump, Sdl, VideoSubsystem,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::Window as SDLWindow,
};
use stellar_app::base::AppState;
use stellar_input::keyboard::{KeyCode, KeyEvent};

use crate::{Window, WindowCloseRequested, WindowResized};

#[derive(Component)]
pub struct SdlWindowId(pub u32);

#[derive(Default)]
pub(crate) struct SdlWindows {
    windows: HashMap<u32, SDLWindow>,
    entity_by_id: HashMap<u32, Entity>,
}

pub(crate) struct SDLState {
    pub(crate) windows: SdlWindows,
    pub(crate) pump: EventPump,
    pub(crate) video: VideoSubsystem,
    pub(crate) _sdl: Sdl,
}

thread_local! {
    static SDL: RefCell<Option<SDLState>> = const { RefCell::new(None) };
}

pub(crate) fn with_sdl<R>(f: impl FnOnce(&mut SDLState) -> R) -> R {
    SDL.with(|cell| {
        let mut guard = cell.borrow_mut();
        let state = guard
            .as_mut()
            .expect("SDL not initialized, or a system ran off the main thread");
        f(state)
    })
}

pub(crate) fn sdl_init(_marker: NonSendMarker) {
    let sdl = sdl3::init().unwrap();
    let video = sdl.video().unwrap();
    let pump = sdl.event_pump().unwrap();

    SDL.with(|cell| {
        *cell.borrow_mut() = Some(SDLState {
            windows: SdlWindows::default(),
            pump,
            video,
            _sdl: sdl,
        });
    });
}

fn pump_events(
    sdl: &mut SDLState,
    mut cmd: Commands,
    mut state: ResMut<AppState>,
    mut key_writer: MessageWriter<KeyEvent>,
) {
    for event in sdl.pump.poll_iter() {
        match event {
            Event::Quit { .. } => state.request(),
            Event::Window {
                window_id,
                win_event,
                ..
            } => {
                let Some(&entity) = sdl.windows.entity_by_id.get(&window_id) else {
                    continue;
                };
                match win_event {
                    WindowEvent::CloseRequested => {
                        cmd.emit(WindowCloseRequested { entity });
                    }
                    WindowEvent::Resized(w, h) => {
                        cmd.emit(WindowResized {
                            entity,
                            width: w as u32,
                            height: h as u32,
                        });
                    }
                    _ => {}
                }
            }
            Event::KeyDown {
                window_id,
                keycode,
                repeat,
                ..
            } => {
                let Some(&entity) = sdl.windows.entity_by_id.get(&window_id) else {
                    continue;
                };

                let Some(keycode) = keycode else {
                    continue;
                };

                key_writer.write(KeyEvent::KeyDown {
                    window: entity,
                    key_code: match_code(keycode),
                    repeat,
                });
            }
            Event::KeyUp {
                window_id,
                keycode,
                repeat,
                ..
            } => {
                let Some(&entity) = sdl.windows.entity_by_id.get(&window_id) else {
                    continue;
                };

                let Some(keycode) = keycode else {
                    continue;
                };

                key_writer.write(KeyEvent::KeyUp {
                    window: entity,
                    key_code: match_code(keycode),
                    repeat,
                });
            }
            _ => {}
        }
    }
}

fn match_code(keycode: Keycode) -> Option<KeyCode> {
    match keycode {
        Keycode::A => Some(KeyCode::KeyA),
        Keycode::B => Some(KeyCode::KeyB),
        Keycode::C => Some(KeyCode::KeyC),
        Keycode::D => Some(KeyCode::KeyD),
        Keycode::E => Some(KeyCode::KeyE),
        Keycode::F => Some(KeyCode::KeyF),
        Keycode::G => Some(KeyCode::KeyG),
        Keycode::H => Some(KeyCode::KeyH),
        Keycode::I => Some(KeyCode::KeyI),
        Keycode::J => Some(KeyCode::KeyJ),
        Keycode::K => Some(KeyCode::KeyK),
        Keycode::L => Some(KeyCode::KeyL),
        Keycode::M => Some(KeyCode::KeyM),
        Keycode::N => Some(KeyCode::KeyN),
        Keycode::O => Some(KeyCode::KeyO),
        Keycode::P => Some(KeyCode::KeyP),
        Keycode::Q => Some(KeyCode::KeyQ),
        Keycode::R => Some(KeyCode::KeyR),
        Keycode::S => Some(KeyCode::KeyS),
        Keycode::T => Some(KeyCode::KeyT),
        Keycode::U => Some(KeyCode::KeyU),
        Keycode::V => Some(KeyCode::KeyV),
        Keycode::W => Some(KeyCode::KeyW),
        Keycode::X => Some(KeyCode::KeyX),
        Keycode::Y => Some(KeyCode::KeyY),
        Keycode::Z => Some(KeyCode::KeyZ),

        Keycode::_0 => Some(KeyCode::Digit0),
        Keycode::_1 => Some(KeyCode::Digit1),
        Keycode::_2 => Some(KeyCode::Digit2),
        Keycode::_3 => Some(KeyCode::Digit3),
        Keycode::_4 => Some(KeyCode::Digit4),
        Keycode::_5 => Some(KeyCode::Digit5),
        Keycode::_6 => Some(KeyCode::Digit6),
        Keycode::_7 => Some(KeyCode::Digit7),
        Keycode::_8 => Some(KeyCode::Digit8),
        Keycode::_9 => Some(KeyCode::Digit9),

        Keycode::F1 => Some(KeyCode::F1),
        Keycode::F2 => Some(KeyCode::F2),
        Keycode::F3 => Some(KeyCode::F3),
        Keycode::F4 => Some(KeyCode::F4),
        Keycode::F5 => Some(KeyCode::F5),
        Keycode::F6 => Some(KeyCode::F6),
        Keycode::F7 => Some(KeyCode::F7),
        Keycode::F8 => Some(KeyCode::F8),
        Keycode::F9 => Some(KeyCode::F9),
        Keycode::F10 => Some(KeyCode::F10),
        Keycode::F11 => Some(KeyCode::F11),
        Keycode::F12 => Some(KeyCode::F12),

        Keycode::Escape => Some(KeyCode::Escape),
        Keycode::Delete => Some(KeyCode::Delete),
        Keycode::Home => Some(KeyCode::Home),
        Keycode::End => Some(KeyCode::End),
        Keycode::Insert => Some(KeyCode::Insert),
        Keycode::Backspace => Some(KeyCode::Backspace),
        Keycode::Tab => Some(KeyCode::Tab),
        Keycode::Space => Some(KeyCode::Space),
        Keycode::PageUp => Some(KeyCode::PageUp),
        Keycode::PageDown => Some(KeyCode::PageDown),
        Keycode::PrintScreen => Some(KeyCode::PrintScreen),
        Keycode::Pause => Some(KeyCode::Pause),

        Keycode::Grave => Some(KeyCode::Backquote),
        Keycode::Backslash => Some(KeyCode::Backslash),
        Keycode::LeftBracket => Some(KeyCode::BracketLeft),
        Keycode::RightBracket => Some(KeyCode::BracketRight),
        Keycode::Comma => Some(KeyCode::Comma),
        Keycode::Period => Some(KeyCode::Period),
        Keycode::Colon => Some(KeyCode::Colon),
        Keycode::Semicolon => Some(KeyCode::Semicolon),
        Keycode::Exclaim => Some(KeyCode::Exclaim),
        Keycode::Asterisk => Some(KeyCode::Asterix),
        Keycode::Question => Some(KeyCode::Question),
        Keycode::Less => Some(KeyCode::Less),
        Keycode::Equals => Some(KeyCode::Equal),
        Keycode::Greater => Some(KeyCode::Greater),
        Keycode::Minus => Some(KeyCode::Minus),
        Keycode::Plus => Some(KeyCode::Plus),
        Keycode::Apostrophe => Some(KeyCode::Quote),

        Keycode::LShift => Some(KeyCode::ShiftLeft),
        Keycode::RShift => Some(KeyCode::ShiftRight),
        Keycode::LCtrl => Some(KeyCode::ControlLeft),
        Keycode::RCtrl => Some(KeyCode::ControlRight),
        Keycode::LAlt => Some(KeyCode::AltLeft),
        Keycode::RAlt => Some(KeyCode::AltRight),
        Keycode::LGui => Some(KeyCode::MetaLeft),
        Keycode::RGui => Some(KeyCode::MetaRight),

        Keycode::CapsLock => Some(KeyCode::CapsLock),
        Keycode::NumLockClear => Some(KeyCode::NumLock),
        Keycode::ScrollLock => Some(KeyCode::ScrollLock),
        Keycode::Application => Some(KeyCode::ContextMenu),

        Keycode::Kp0 => Some(KeyCode::Numpad0),
        Keycode::Kp1 => Some(KeyCode::Numpad1),
        Keycode::Kp2 => Some(KeyCode::Numpad2),
        Keycode::Kp3 => Some(KeyCode::Numpad3),
        Keycode::Kp4 => Some(KeyCode::Numpad4),
        Keycode::Kp5 => Some(KeyCode::Numpad5),
        Keycode::Kp6 => Some(KeyCode::Numpad6),
        Keycode::Kp7 => Some(KeyCode::Numpad7),
        Keycode::Kp8 => Some(KeyCode::Numpad8),
        Keycode::Kp9 => Some(KeyCode::Numpad9),
        Keycode::KpPlus => Some(KeyCode::NumpadAdd),
        Keycode::KpMinus => Some(KeyCode::NumpadSubtract),
        Keycode::KpMultiply => Some(KeyCode::NumpadMultiply),
        Keycode::KpDivide => Some(KeyCode::NumpadDivide),
        Keycode::KpPeriod => Some(KeyCode::NumpadDecimal),
        Keycode::KpEnter => Some(KeyCode::NumpadEnter),
        Keycode::KpEquals => Some(KeyCode::NumpadEqual),

        Keycode::Right => Some(KeyCode::Right),
        Keycode::Left => Some(KeyCode::Left),
        Keycode::Up => Some(KeyCode::Up),
        Keycode::Down => Some(KeyCode::Down),

        Keycode::Mute => Some(KeyCode::AudioVolumeMute),
        Keycode::VolumeUp => Some(KeyCode::AudioVolumeUp),
        Keycode::VolumeDown => Some(KeyCode::AudioVolumeDown),
        Keycode::MediaPlayPause => Some(KeyCode::MediaPlayPause),
        Keycode::MediaStop => Some(KeyCode::MediaStop),
        Keycode::MediaNextTrack => Some(KeyCode::MediaTrackNext),
        Keycode::MediaPreviousTrack => Some(KeyCode::MediaTrackPrevious),

        _ => Some(KeyCode::Unidentified),
    }
}

fn sync_windows(
    sdl: &mut SDLState,
    mut cmd: Commands,
    alive: Query<&SdlWindowId>,
    new: Query<(Entity, &Window), Without<SdlWindowId>>,
) {
    let alive_ids: HashSet<u32> = alive.iter().map(|id| id.0).collect();
    sdl.windows.windows.retain(|id, _| alive_ids.contains(id));
    sdl.windows
        .entity_by_id
        .retain(|id, _| alive_ids.contains(id));

    for (entity, w) in new.iter() {
        let mut builder = sdl.video.window(&w.title, w.inner_size.0, w.inner_size.0);
        if w.resizable {
            builder.resizable();
        }
        let win = builder.build().unwrap();
        let id = win.id();

        sdl.windows.entity_by_id.insert(id, entity);
        sdl.windows.windows.insert(id, win);
        cmd.insert_component(entity, SdlWindowId(id));

        info!("Created window: {}", w.title);
    }
}

pub(crate) fn sdl_frame_begin(
    cmd: Commands,
    cmd2: Commands,
    state: ResMut<AppState>,
    alive: Query<&SdlWindowId>,
    new: Query<(Entity, &Window), Without<SdlWindowId>>,
    key_writer: MessageWriter<KeyEvent>,
    _marker: NonSendMarker,
) {
    with_sdl(|sdl| {
        sync_windows(sdl, cmd, alive, new);
        pump_events(sdl, cmd2, state, key_writer);
    });
}

pub(crate) fn sdl_drop(_marker: NonSendMarker) {
    SDL.with(|cell| *cell.borrow_mut() = None)
}
