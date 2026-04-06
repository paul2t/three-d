/// Event recording and replay for the winit render loop.
///
/// # Environment variables
///
/// - `RECORD_EVENTS=path/to/file.json`  — record all winit events to a newline-delimited JSON file.
/// - `REPLAY_EVENTS=path/to/file.json`  — replay events from that file, replacing the real event loop.
///
/// If both are set, `RECORD_EVENTS` takes precedence.
///
/// # File format
///
/// One JSON object per line (NDJSON). Each line is a [`RecordedEvent`].
///
/// # Usage in render_loop
///
/// ```rust
/// let mut recorder = EventRecorder::from_env();
///
/// self.event_loop.run(move |event, event_loop| {
///     // In replay mode, the real `event` is ignored and the next
///     // recorded event is used instead (mapped back to a winit Event).
///     let event = recorder.process(&event);
///     let Some(event) = event else { return };
///
///     match event { ... } // your existing match arms unchanged
/// });
/// ```
use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    keyboard::PhysicalKey,
};

// ─── Serialisable event mirror ────────────────────────────────────────────────

/// A serialisable mirror of the winit `Event` variants that are relevant to
/// record/replay. Variants that carry OS handles or are not reproducible
/// (e.g. `DeviceEvent`, `UserEvent`) are omitted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
#[allow(missing_docs)]
pub enum RecordedEvent {
    // ── Loop lifecycle ────────────────────────────────────────────────────────
    AboutToWait,
    LoopExiting,
    Suspended,
    Resumed,
    MemoryWarning,

    // ── WindowEvent wrappers ──────────────────────────────────────────────────
    Resized {
        width: u32,
        height: u32,
    },
    ScaleFactorChanged {
        scale_factor: f64,
    },
    Occluded {
        visible: bool,
    },
    CloseRequested,
    RedrawRequested,
    KeyboardInput {
        key_code: String,
        state: String,
        /// Printable text produced by this key, if any.
        text: Option<String>,
    },
    MouseWheel {
        /// `"line"` or `"pixel"`
        delta_type: String,
        x: f64,
        y: f64,
    },
    PinchGesture {
        delta: f64,
    },
    RotationGesture {
        /// radians
        delta: f32,
    },
    MouseInput {
        /// `"pressed"` | `"released"`
        state: String,
        /// `"left"` | `"middle"` | `"right"` | `"other"`
        button: String,
    },
    CursorMoved {
        /// Physical pixels
        x: f64,
        y: f64,
    },
    CursorEntered,
    CursorLeft,
    Touch {
        /// `"started"` | `"moved"` | `"ended"` | `"cancelled"`
        phase: String,
        id: u64,
        /// Physical pixels
        x: f64,
        y: f64,
    },
    HoveredFile {
        path: String,
    },
    HoveredFileCancelled,
    DroppedFile {
        path: String,
    },
}

// ─── Conversion: winit Event → RecordedEvent ──────────────────────────────────

/// Convert a winit `Event<T>` to a `RecordedEvent`, returning `None` for
/// variants we don't record (e.g. `DeviceEvent`, `UserEvent`, `NewEvents`).
pub fn winit_event_to_recorded<T>(event: &winit::event::Event<T>) -> Option<RecordedEvent> {
    use winit::event::Event;

    match event {
        Event::AboutToWait => Some(RecordedEvent::AboutToWait),
        Event::LoopExiting => Some(RecordedEvent::LoopExiting),
        Event::Suspended => Some(RecordedEvent::Suspended),
        Event::Resumed => Some(RecordedEvent::Resumed),
        Event::MemoryWarning => Some(RecordedEvent::MemoryWarning),
        Event::WindowEvent { event, .. } => window_event_to_recorded(event),
        // DeviceEvent, UserEvent, NewEvents are intentionally skipped.
        _ => None,
    }
}

fn window_event_to_recorded(event: &WindowEvent) -> Option<RecordedEvent> {
    match event {
        WindowEvent::Resized(size) => Some(RecordedEvent::Resized {
            width: size.width,
            height: size.height,
        }),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(RecordedEvent::ScaleFactorChanged {
                scale_factor: *scale_factor,
            })
        }
        WindowEvent::Occluded(visible) => Some(RecordedEvent::Occluded { visible: *visible }),
        WindowEvent::CloseRequested => Some(RecordedEvent::CloseRequested),
        WindowEvent::RedrawRequested => Some(RecordedEvent::RedrawRequested),
        WindowEvent::KeyboardInput { event, .. } => {
            let key_code = match event.physical_key {
                PhysicalKey::Code(code) => format!("{:?}", code),
                PhysicalKey::Unidentified(k) => format!("Unidentified({:?})", k),
            };
            let state = match event.state {
                ElementState::Pressed => "pressed",
                ElementState::Released => "released",
            };
            let text = event.text.as_ref().map(|s| s.to_string());
            Some(RecordedEvent::KeyboardInput {
                key_code,
                state: state.to_string(),
                text,
            })
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            MouseScrollDelta::LineDelta(x, y) => Some(RecordedEvent::MouseWheel {
                delta_type: "line".to_string(),
                x: *x as f64,
                y: *y as f64,
            }),
            MouseScrollDelta::PixelDelta(pos) => Some(RecordedEvent::MouseWheel {
                delta_type: "pixel".to_string(),
                x: pos.x,
                y: pos.y,
            }),
        },
        WindowEvent::PinchGesture { delta, .. } => {
            Some(RecordedEvent::PinchGesture { delta: *delta })
        }
        WindowEvent::RotationGesture { delta, .. } => {
            Some(RecordedEvent::RotationGesture { delta: *delta })
        }
        WindowEvent::MouseInput { state, button, .. } => {
            let button_str = match button {
                MouseButton::Left => "left",
                MouseButton::Middle => "middle",
                MouseButton::Right => "right",
                _ => "other",
            };
            let state_str = match state {
                ElementState::Pressed => "pressed",
                ElementState::Released => "released",
            };
            Some(RecordedEvent::MouseInput {
                state: state_str.to_string(),
                button: button_str.to_string(),
            })
        }
        WindowEvent::CursorMoved { position, .. } => Some(RecordedEvent::CursorMoved {
            x: position.x,
            y: position.y,
        }),
        WindowEvent::CursorEntered { .. } => Some(RecordedEvent::CursorEntered),
        WindowEvent::CursorLeft { .. } => Some(RecordedEvent::CursorLeft),
        WindowEvent::Touch(touch) => {
            let phase = match touch.phase {
                TouchPhase::Started => "started",
                TouchPhase::Moved => "moved",
                TouchPhase::Ended => "ended",
                TouchPhase::Cancelled => "cancelled",
            };
            Some(RecordedEvent::Touch {
                phase: phase.to_string(),
                id: touch.id,
                x: touch.location.x,
                y: touch.location.y,
            })
        }
        WindowEvent::HoveredFile(path) => Some(RecordedEvent::HoveredFile {
            path: path.to_string_lossy().to_string(),
        }),
        WindowEvent::HoveredFileCancelled => Some(RecordedEvent::HoveredFileCancelled),
        WindowEvent::DroppedFile(path) => Some(RecordedEvent::DroppedFile {
            path: path.to_string_lossy().to_string(),
        }),
        _ => None,
    }
}

// ─── Conversion: RecordedEvent → synthetic winit Event ────────────────────────

/// Reconstruct a winit `Event<T>` from a `RecordedEvent`.
///
/// Some fields that winit normally fills from the OS (e.g. `WindowId`,
/// `DeviceId`, repeat counts) are given placeholder values — they are ignored
/// by the existing match arms that only pattern-match on the inner payload.
pub fn recorded_to_winit_event<T>(recorded: &RecordedEvent) -> Option<winit::event::Event<T>> {
    use winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{
            ElementState, KeyEvent, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent,
        },
        keyboard::{Key, NativeKeyCode, PhysicalKey},
        window::WindowId,
    };

    // Safety: winit provides no stable way to construct WindowId / DeviceId
    // from scratch. We use a zeroed placeholder; the existing code never reads
    // these ids — it only matches on the inner event payload.
    let dummy_window_id: WindowId = unsafe { std::mem::zeroed() };

    let wrap = |we: WindowEvent| winit::event::Event::WindowEvent {
        window_id: dummy_window_id,
        event: we,
    };

    match recorded {
        // ── Loop lifecycle ────────────────────────────────────────────────────
        RecordedEvent::AboutToWait => Some(winit::event::Event::AboutToWait),
        RecordedEvent::LoopExiting => Some(winit::event::Event::LoopExiting),
        RecordedEvent::Suspended => Some(winit::event::Event::Suspended),
        RecordedEvent::Resumed => Some(winit::event::Event::Resumed),
        RecordedEvent::MemoryWarning => Some(winit::event::Event::MemoryWarning),

        // ── WindowEvents ──────────────────────────────────────────────────────
        RecordedEvent::Resized { width, height } => Some(wrap(WindowEvent::Resized(
            PhysicalSize::new(*width, *height),
        ))),
        RecordedEvent::ScaleFactorChanged { scale_factor } => {
            // new_inner_size field removed in winit 0.30 — pass only scale_factor.
            Some(wrap(WindowEvent::ScaleFactorChanged {
                scale_factor: *scale_factor,
                inner_size_writer: unsafe { std::mem::zeroed() },
            }))
        }
        RecordedEvent::Occluded { visible } => Some(wrap(WindowEvent::Occluded(*visible))),
        RecordedEvent::CloseRequested => Some(wrap(WindowEvent::CloseRequested)),
        RecordedEvent::RedrawRequested => Some(wrap(WindowEvent::RedrawRequested)),

        RecordedEvent::KeyboardInput {
            key_code,
            state,
            text,
        } => {
            let physical_key = serde_json::from_value(serde_json::Value::String(key_code.clone()))
                .map(PhysicalKey::Code)
                .unwrap_or(PhysicalKey::Unidentified(NativeKeyCode::Unidentified));
            let key_event: KeyEvent = unsafe {
                let mut e: KeyEvent = std::mem::zeroed();
                e.physical_key = physical_key;
                e.logical_key = Key::Unidentified(winit::keyboard::NativeKey::Unidentified);
                e.text = text.as_deref().map(winit::keyboard::SmolStr::new);
                e.location = winit::keyboard::KeyLocation::Standard;
                e.state = if state == "pressed" {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };
                e.repeat = false;
                e
            };
            Some(wrap(WindowEvent::KeyboardInput {
                device_id: unsafe { std::mem::zeroed() },
                event: key_event,
                is_synthetic: true,
            }))
        }

        RecordedEvent::MouseWheel { delta_type, x, y } => {
            let delta = if delta_type == "line" {
                MouseScrollDelta::LineDelta(*x as f32, *y as f32)
            } else {
                MouseScrollDelta::PixelDelta(PhysicalPosition::new(*x, *y))
            };
            Some(wrap(WindowEvent::MouseWheel {
                device_id: unsafe { std::mem::zeroed() },
                delta,
                phase: winit::event::TouchPhase::Moved,
            }))
        }

        RecordedEvent::PinchGesture { delta } => Some(wrap(WindowEvent::PinchGesture {
            device_id: unsafe { std::mem::zeroed() },
            delta: *delta,
            phase: TouchPhase::Moved,
        })),

        RecordedEvent::RotationGesture { delta } => Some(wrap(WindowEvent::RotationGesture {
            device_id: unsafe { std::mem::zeroed() },
            delta: *delta,
            phase: TouchPhase::Moved,
        })),

        RecordedEvent::MouseInput { state, button } => {
            let button = match button.as_str() {
                "left" => MouseButton::Left,
                "middle" => MouseButton::Middle,
                "right" => MouseButton::Right,
                _ => MouseButton::Other(0),
            };
            let state = if state == "pressed" {
                ElementState::Pressed
            } else {
                ElementState::Released
            };
            Some(wrap(WindowEvent::MouseInput {
                device_id: unsafe { std::mem::zeroed() },
                state,
                button,
            }))
        }

        RecordedEvent::CursorMoved { x, y } => Some(wrap(WindowEvent::CursorMoved {
            device_id: unsafe { std::mem::zeroed() },
            position: PhysicalPosition::new(*x, *y),
        })),

        RecordedEvent::CursorEntered => Some(wrap(WindowEvent::CursorEntered {
            device_id: unsafe { std::mem::zeroed() },
        })),

        RecordedEvent::CursorLeft => Some(wrap(WindowEvent::CursorLeft {
            device_id: unsafe { std::mem::zeroed() },
        })),

        RecordedEvent::Touch { phase, id, x, y } => {
            let phase = match phase.as_str() {
                "started" => TouchPhase::Started,
                "moved" => TouchPhase::Moved,
                "ended" => TouchPhase::Ended,
                _ => TouchPhase::Cancelled,
            };
            Some(wrap(WindowEvent::Touch(Touch {
                device_id: unsafe { std::mem::zeroed() },
                phase,
                location: PhysicalPosition::new(*x, *y),
                force: None,
                id: *id,
            })))
        }

        RecordedEvent::HoveredFile { path } => {
            Some(wrap(WindowEvent::HoveredFile(PathBuf::from(path))))
        }
        RecordedEvent::HoveredFileCancelled => Some(wrap(WindowEvent::HoveredFileCancelled)),
        RecordedEvent::DroppedFile { path } => {
            Some(wrap(WindowEvent::DroppedFile(PathBuf::from(path))))
        }
    }
}

// ─── EventRecorder ────────────────────────────────────────────────────────────

/// Handles recording and replaying winit events.
///
/// Initialise once with [`EventRecorder::from_env`], then call
/// [`EventRecorder::process`] for every event in the render loop.
pub struct EventRecorder {
    mode: Mode,
}

enum Mode {
    Passthrough,
    Record {
        writer: File,
    },
    Replay {
        events: Vec<RecordedEvent>,
        /// Index of the next event to emit.
        index: usize,
    },
}

impl EventRecorder {
    /// Read `RECORD_EVENTS` / `REPLAY_EVENTS` from the environment and return
    /// the appropriate recorder. Panics if the specified file cannot be opened.
    pub fn from_env() -> Self {
        if let Ok(path) = env::var("RECORD_EVENTS") {
            let writer = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .unwrap_or_else(|e| panic!("RECORD_EVENTS: cannot open '{path}': {e}"));
            eprintln!("[event-recorder] Recording to: {path}");
            Self {
                mode: Mode::Record { writer },
            }
        } else if let Ok(path) = env::var("REPLAY_EVENTS") {
            let file = File::open(&path)
                .unwrap_or_else(|e| panic!("REPLAY_EVENTS: cannot open '{path}': {e}"));
            let events: Vec<RecordedEvent> = BufReader::new(file)
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    let line = line.expect("IO error reading replay file");
                    let line = line.trim();
                    if line.is_empty() {
                        return None;
                    }
                    serde_json::from_str(line)
                        .map_err(|e| eprintln!("[event-replay] Skipping line {i}: {e}"))
                        .ok()
                })
                .collect();
            eprintln!(
                "[event-replay] Replaying {} events from: {path}",
                events.len()
            );
            Self {
                mode: Mode::Replay { events, index: 0 },
            }
        } else {
            Self {
                mode: Mode::Passthrough,
            }
        }
    }

    /// Process one event from the winit event loop.
    ///
    /// **Record mode** — serialises the real event to disk and returns it
    /// unchanged so your existing handler runs normally.
    ///
    /// **Replay mode** — ignores the real event entirely. Returns the next
    /// recorded event (reconstructed as a winit `Event`), or `None` when the
    /// recording is exhausted (you should call `event_loop.exit()` then).
    ///
    /// **Passthrough mode** — returns the real event unchanged.
    pub fn process<T: 'static + Clone>(
        &mut self,
        event: &winit::event::Event<T>,
    ) -> Option<winit::event::Event<T>> {
        match &mut self.mode {
            Mode::Passthrough => Some(event.clone()),

            Mode::Record { writer } => {
                if let Some(recorded) = winit_event_to_recorded(event) {
                    let mut json = serde_json::to_string(&recorded)
                        .expect("RecordedEvent serialisation failed");
                    json.push('\n');
                    writer
                        .write_all(json.as_bytes())
                        .expect("Failed to write event to record file");
                }
                Some(event.clone())
            }

            Mode::Replay { events, index } => {
                // Skip real events completely.
                if *index >= events.len() {
                    // Recording exhausted — signal the caller.
                    return None;
                }
                let recorded = &events[*index];
                *index += 1;
                // Reconstruct a winit Event from the recording.
                // Falls back to None for variants we can't reconstruct
                // (shouldn't happen for anything we record).
                recorded_to_winit_event(recorded)
            }
        }
    }

    /// `true` when replay has consumed all recorded events.
    pub fn replay_finished(&self) -> bool {
        match &self.mode {
            Mode::Replay { events, index } => *index >= events.len(),
            _ => false,
        }
    }

    /// `true` when replay in progress.
    pub fn is_replaying(&self) -> bool {
        matches!(self.mode, Mode::Replay { .. })
    }

    /// `true` when record in progress.
    pub fn is_recording(&self) -> bool {
        matches!(self.mode, Mode::Record { .. })
    }
}
