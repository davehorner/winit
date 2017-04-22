//! Winit allows you to build a window on as many platforms as possible. 
//!
//! # Building a window
//!
//! Before you can build a window, you first need to build an `EventsLoop`. This is done with the
//! `EventsLoop::new()` function. Example:
//!
//! ```no_run
//! use winit::EventsLoop;
//! let events_loop = EventsLoop::new();
//! ```
//!
//! Once this is done there are two ways to create a window:
//!
//!  - Calling `Window::new(&events_loop)`.
//!  - Calling `let builder = WindowBuilder::new()` then `builder.build(&events_loop)`.
//!
//! The first way is the simpliest way and will give you default values for everything.
//!
//! The second way allows you to customize the way your window will look and behave by modifying
//! the fields of the `WindowBuilder` object before you create the window.
//!
//! # Events handling
//!
//! Once a window has been created, it will *generate events*. For example whenever the user moves
//! the window, resizes the window, moves the mouse, etc. an event is generated.
//!
//! The events generated by a window can be retreived from the `EventsLoop` the window was created
//! with.
//!
//! There are two ways to do so. The first is to call `events_loop.poll_events(...)`, which will
//! retreive all the events pending on the windows and immediately return after no new event is
//! available. You usually want to use this method in application that render continuously on the
//! screen, such as video games.
//!
//! ```no_run
//! use winit::Event;
//! use winit::WindowEvent;
//! # use winit::EventsLoop;
//! # let events_loop = EventsLoop::new();
//!
//! loop {
//!     events_loop.poll_events(|event| {
//!         match event {
//!             Event::WindowEvent { event: WindowEvent::Resized(w, h), .. } => {
//!                 println!("The window was resized to {}x{}", w, h);
//!             },
//!             _ => ()
//!         }
//!     });
//! }
//! ```
//!
//! The second way is to call `events_loop.run_forever(...)`. As its name tells, it will run
//! forever unless it is stopped by calling `events_loop.interrupt()`.
//!
//! ```no_run
//! use winit::Event;
//! use winit::WindowEvent;
//! # use winit::EventsLoop;
//! # let events_loop = EventsLoop::new();
//!
//! events_loop.run_forever(|event| {
//!     match event {
//!         Event::WindowEvent { event: WindowEvent::Closed, .. } => {
//!             println!("The window was closed ; stopping");
//!             events_loop.interrupt();
//!         },
//!         _ => ()
//!     }
//! });
//! ```
//!
//! If you use multiple windows, the `WindowEvent` event has a member named `window_id`. You can
//! compare it with the value returned by the `id()` method of `Window` in order to know which
//! window has received the event.
//!
//! # Drawing on the window
//!
//! Winit doesn't provide any function that allows drawing on a window. However it allows you to
//! retreive the raw handle of the window (see the `os` module for that), which in turn allows you
//! to create an OpenGL/Vulkan/DirectX/Metal/etc. context that will draw on the window.
//!

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate shared_library;

extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate shell32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate dwmapi;
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;
#[cfg(target_os = "macos")]
extern crate cgl;
#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]
extern crate x11_dl;
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd"))]
#[macro_use(wayland_env,declare_handler)]
extern crate wayland_client;

use std::sync::Arc;

pub use events::*;
pub use window::{AvailableMonitorsIter, MonitorId, get_available_monitors, get_primary_monitor};
pub use native_monitor::NativeMonitorId;

#[macro_use]
mod api_transition;

mod platform;
mod events;
mod window;

pub mod os;

/// Represents a window.
///
/// # Example
///
/// ```no_run
/// use winit::Event;
/// use winit::EventsLoop;
/// use winit::Window;
/// use winit::WindowEvent;
///
/// let events_loop = EventsLoop::new();
/// let window = Window::new(&events_loop).unwrap();
///
/// events_loop.run_forever(|event| {
///     match event {
///         Event::WindowEvent { event: WindowEvent::Closed, .. } => {
///             events_loop.interrupt();
///         },
///         _ => ()
///     }
/// });
/// ```
pub struct Window {
    window: platform::Window2,
}

/// Identifier of a window. Unique for each window.
///
/// Can be obtained with `window.id()`.
///
/// Whenever you receive an event specific to a window, this event contains a `WindowId` which you
/// can then compare to the ids of your windows.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(platform::WindowId);

/// Identifier of an input device.
///
/// Whenever you receive an event arising from a particular input device, this event contains a `DeviceId` which
/// identifies its origin. Note that devices may be virtual (representing an on-screen cursor and keyboard focus) or
/// physical. Virtual devices typically aggregate inputs from multiple physical devices.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(platform::DeviceId);

/// Identifier for a specific analog axis on some device.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxisId(u32);

/// Identifier for a specific button on some device.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ButtonId(u32);

/// Provides a way to retreive events from the windows that were registered to it.
// TODO: document usage in multiple threads
pub struct EventsLoop {
    events_loop: Arc<platform::EventsLoop>,
}

impl EventsLoop {
    /// Builds a new events loop.
    pub fn new() -> EventsLoop {
        EventsLoop {
            events_loop: Arc::new(platform::EventsLoop::new()),
        }
    }

    /// Fetches all the events that are pending, calls the callback function for each of them,
    /// and returns.
    #[inline]
    pub fn poll_events<F>(&self, callback: F)
        where F: FnMut(Event)
    {
        self.events_loop.poll_events(callback)
    }

    /// Runs forever until `interrupt()` is called. Whenever an event happens, calls the callback.
    #[inline]
    pub fn run_forever<F>(&self, callback: F)
        where F: FnMut(Event)
    {
        self.events_loop.run_forever(callback)
    }

    /// If we called `run_forever()`, stops the process of waiting for events.
    // TODO: what if we're waiting from multiple threads?
    #[inline]
    pub fn interrupt(&self) {
        self.events_loop.interrupt()
    }
}

/// Object that allows you to build windows.
#[derive(Clone)]
pub struct WindowBuilder {
    /// The attributes to use to create the window.
    pub window: WindowAttributes,

    // Platform-specific configuration. Private.
    platform_specific: platform::PlatformSpecificWindowBuilderAttributes,
}

/// Error that can happen while creating a window or a headless renderer.
#[derive(Debug)]
pub enum CreationError {
    OsError(String),
    /// TODO: remove this error
    NotSupported,
}

impl CreationError {
    fn to_string(&self) -> &str {
        match *self {
            CreationError::OsError(ref text) => &text,
            CreationError::NotSupported => "Some of the requested attributes are not supported",
        }
    }
}

impl std::fmt::Display for CreationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str(self.to_string())
    }
}

impl std::error::Error for CreationError {
    fn description(&self) -> &str {
        self.to_string()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseCursor {
    /// The platform-dependent default cursor.
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// Self explanatory.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    NoneCursor,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

/// Describes how glutin handles the cursor.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CursorState {
    /// Normal cursor behavior.
    Normal,

    /// The cursor will be invisible when over the window.
    Hide,

    /// Grabs the mouse cursor. The cursor's motion will be confined to this
    /// window and the window has exclusive access to further events regarding
    /// the cursor.
    ///
    /// This is useful for first-person cameras for example.
    Grab,
}

/// Attributes to use when creating a window.
#[derive(Clone)]
pub struct WindowAttributes {
    /// The dimensions of the window. If this is `None`, some platform-specific dimensions will be
    /// used.
    ///
    /// The default is `None`.
    pub dimensions: Option<(u32, u32)>,

    /// The minimum dimensions a window can be, If this is `None`, the window will have no minimum dimensions (aside from reserved).
    ///
    /// The default is `None`.
    pub min_dimensions: Option<(u32, u32)>,

    /// The maximum dimensions a window can be, If this is `None`, the maximum will have no maximum or will be set to the primary monitor's dimensions by the platform.
    ///
    /// The default is `None`.
    pub max_dimensions: Option<(u32, u32)>,

    /// If `Some`, the window will be in fullscreen mode with the given monitor.
    ///
    /// The default is `None`.
    pub monitor: Option<platform::MonitorId>,

    /// The title of the window in the title bar.
    ///
    /// The default is `"glutin window"`.
    pub title: String,

    /// Whether the window should be immediately visible upon creation.
    ///
    /// The default is `true`.
    pub visible: bool,

    /// Whether the the window should be transparent. If this is true, writing colors
    /// with alpha values different than `1.0` will produce a transparent window.
    ///
    /// The default is `false`.
    pub transparent: bool,

    /// Whether the window should have borders and bars.
    ///
    /// The default is `true`.
    pub decorations: bool,

    /// [iOS only] Enable multitouch, see [UIView#multipleTouchEnabled]
    /// (https://developer.apple.com/library/ios/documentation/UIKit/Reference/UIView_Class/#//apple_ref/occ/instp/UIView/multipleTouchEnabled)
    pub multitouch: bool,
}

impl Default for WindowAttributes {
    #[inline]
    fn default() -> WindowAttributes {
        WindowAttributes {
            dimensions: None,
            min_dimensions: None,
            max_dimensions: None,
            monitor: None,
            title: "glutin window".to_owned(),
            visible: true,
            transparent: false,
            decorations: true,
            multitouch: false,
        }
    }
}

mod native_monitor {
    /// Native platform identifier for a monitor. Different platforms use fundamentally different types
    /// to represent a monitor ID.
    #[derive(Clone, PartialEq, Eq)]
    pub enum NativeMonitorId {
        /// Cocoa and X11 use a numeric identifier to represent a monitor.
        Numeric(u32),

        /// Win32 uses a Unicode string to represent a monitor.
        Name(String),

        /// Other platforms (Android) don't support monitor identification.
        Unavailable
    }
}
