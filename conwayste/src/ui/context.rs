/*  Copyright 2019 the Conwayste Developers.
 *
 *  This file is part of conwayste.
 *
 *  conwayste is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  conwayste is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with conwayste.  If not, see
 *  <http://www.gnu.org/licenses/>. */

use std::collections::HashMap;
use std::error::Error;

use downcast_rs::Downcast;
use enum_iterator::IntoEnumIterator;
use ggez;
use ggez::event::MouseButton;
use ggez::nalgebra::Point2;

use crate::config;

pub enum UIContext<'a> {
    Draw(DrawContext<'a>),
    Update(UpdateContext<'a>),
}

impl<'a> UIContext<'a> {
    #[allow(dead_code)]
    pub fn unwrap_draw(&mut self) -> &mut DrawContext<'a> {
        match *self {
            UIContext::Draw(ref mut draw_context) => draw_context,
            _ => panic!("Failed to unwrap DrawContext"),
        }
    }

    pub fn unwrap_update(&mut self) -> &mut UpdateContext<'a> {
        match *self {
            UIContext::Update(ref mut update_context) => update_context,
            _ => panic!("Failed to unwrap UpdateContext"),
        }
    }

    #[allow(dead_code)]
    pub fn new_draw(ggez_context: &'a mut ggez::Context, config: &'a config::Config) -> Self {
        UIContext::Draw(DrawContext {
            ggez_context,
            config,
        })
    }

    pub fn new_update(ggez_context: &'a mut ggez::Context, config: &'a mut config::Config) -> Self {
        UIContext::Update(UpdateContext {
            ggez_context,
            config,
        })
    }
}

pub struct DrawContext<'a> {
    pub ggez_context: &'a mut ggez::Context,
    pub config: &'a config::Config,
}

pub struct UpdateContext<'a> {
    pub ggez_context: &'a mut ggez::Context,
    pub config: &'a mut config::Config,
}

/// The type of an event.
#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum EventType {
    Click,
    KeyPress,
    MouseMove,
    Drag,
    Translate,
    Resize,
    ParentTranslate,
    ParentResize,
    // TODO: not sure about Child* because we'd need a widget ID to say which child
    //ChildTranslate,
    //ChildResize,
}

// TODO: move this elsewhere; it's in here to keep separate from other code (avoid merge conflicts)
#[derive(Debug, Clone)]
pub struct Event {
    pub what: EventType,
    pub point: Point2<f32>, // Must not be None if this is a mouse event type
    pub prev_point: Option<Point2<f32>>, // MouseMove / Drag
    pub button: Option<MouseButton>, // Click
}

/// A slice containing all EventTypes related to the keyboard.
pub const KEY_EVENTS: &[EventType] = &[EventType::KeyPress];

/// A slice containing all EventTypes related to the mouse.
pub const MOUSE_EVENTS: &[EventType] = &[EventType::Click, EventType::MouseMove, EventType::Drag];

impl EventType {
    /// Returns true if and only if this is a keyboard event type.
    pub fn is_key_event(self) -> bool {
        KEY_EVENTS.contains(&self)
    }

    /// Returns true if and only if this is a mouse event type. This implies that point is valid.
    pub fn is_mouse_event(self) -> bool {
        MOUSE_EVENTS.contains(&self)
    }
}

impl Event {
    /// Returns true if and only if this is a keyboard event.
    pub fn is_key_event(&self) -> bool {
        self.what.is_key_event()
    }

    /// Returns true if and only if this is a mouse event. This implies that point is valid.
    pub fn is_mouse_event(&self) -> bool {
        self.what.is_mouse_event()
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Handled {
    Handled,    // no other handlers should be called
    NotHandled, // continue calling handlers
}

pub type Handler = Box<
    dyn FnMut(&mut dyn EmitEvent, &mut UIContext, &Event) -> Result<Handled, Box<dyn Error>> + Send,
>;

pub type HandlerMap = HashMap<EventType, Vec<Handler>>;

/// Trait for widgets that can handle various events. Use `.on` to register a handler and `.emit`
/// to emit an event which will cause all handlers for the event's type to be called.
///
/// # Errors
///
/// * It is an error to call `.emit` or `.on` from within a handler.
pub trait EmitEvent: Downcast {
    /// Setup a handler for an event type
    ///
    /// ```
    /// let handler = |obj: &mut dyn EmitEvent, uictx: &mut context::UIContext, evt: &context::Event| {
    ///     use context::Handled::*;
    ///     let mut widget = obj.downcast_mut::<MyWidget>().unwrap();
    ///
    ///     //... do stuff
    ///
    ///     Ok(Handled) // can also return NotHandled to allow other handlers for this event type to run
    /// };
    /// my_widget.on(context::EventType::Click, Box::new(handler));
    /// ```
    ///
    /// # Errors
    ///
    /// * It is an error to call this from within a handler.
    fn on(&mut self, what: EventType, f: Handler) -> Result<(), Box<dyn Error>>;

    /// Emit an event -- call all handlers for this event's type (as long as they return NotHandled)
    ///
    /// # Errors
    ///
    /// * It is an error to call this from within a handler.
    /// * The first error to be returned by a handler will be returned here, and no other handlers
    ///   will run.
    fn emit(&mut self, event: &Event, uictx: &mut UIContext) -> Result<(), Box<dyn Error>>;
}

impl_downcast!(EmitEvent);

/// Implement EmitEvent for a widget (though strictly speaking non-widgets can implement it).
///
/// # Example
///
/// ```
/// struct MyWidget {
///     handlers: Option<HandlerMap>,
///     ...
/// }
///
/// impl MyWidget {
///     fn new() -> Self {
///         MyWidget {
///             handlers: Some(context::HandlerMap::new()),
///             ...
///         }
///     }
/// }
/// // top level of the module
/// impl_emit_event!(MyWidget, self.handlers);
/// ```
#[macro_export]
macro_rules! impl_emit_event {
    ($widget_name:ty, self.$handler_field:ident) => {
        impl crate::ui::context::EmitEvent for $widget_name {
            /// Setup a handler for an event type
            fn on(&mut self, what: crate::ui::context::EventType, hdlr: crate::ui::context::Handler) -> Result<(), Box<dyn std::error::Error>> {
                let handlers = self.$handler_field
                    .as_mut()
                    .ok_or_else(|| -> Box<dyn std::error::Error> {
                        format!(".on({:?}, ...) was called while .emit call was in progress for {} widget",
                        what,
                        stringify!($widget_name)).into()
                    })?;

                let handler_vec: &mut Vec<crate::ui::context::Handler>;
                if let Some(vref) = handlers.get_mut(&what) {
                    handler_vec = vref;
                } else {
                    handlers.insert(what, vec![]);
                    handler_vec = handlers.get_mut(&what).unwrap();
                }
                handler_vec.push(hdlr);
                Ok(())
            }

            /// Emit an event -- call all handlers for this event's type (as long as they return NotHandled)
            fn emit(&mut self, event: &crate::ui::context::Event, uictx: &mut crate::ui::context::UIContext) -> Result<(), Box<dyn std::error::Error>> {
                use crate::ui::context::Handled::*;
                // HACK: prevent a borrow error when calling handlers
                let mut handlers = self.$handler_field
                    .take()
                    .ok_or_else(|| -> Box<dyn std::error::Error> {
                        format!(".emit({:?}, ...) was called while another .emit call was in progress for {} widget",
                                event.what,
                                stringify!($widget_name)).into()
                    })?;

                if let Some(handler_vec) = handlers.get_mut(&event.what) {
                    // call each handler for this event type, until a Handled is returned
                    for hdlr in handler_vec {
                        let handled = hdlr(self, uictx, event)?;
                        if handled == Handled {
                            break;
                        }
                    }
                }
                self.$handler_field = Some(handlers); // put it back
                Ok(())
            }
        }
    };
}

/// Register a handler for this container widget to dispatch mouse events to whichever contained
/// widget contains the `point` of the event. This will typically be done when creating the widget,
/// in its `new` method.
///
/// # Example
///
/// ```
/// fn new() -> Pane {
///     let my_pane = Pane {
///         ...
///         widgets: vec![],
///         ...
///     };
///     forward_mouse_events!(Pane, my_pane, widgets);
///     my_pane
/// }
/// ```
///
/// # Panics
///
/// This will panic if there is any error returned by the `.on` method of the container widget.
/// Verify that this is still true before relying on it, but at present, this error can only happen
/// if this macro is expanded (and thus, `.on` is called) from within a handler for this container
/// widget.
//TODO: add a forward_keyboard_events! macro that uses focus rather than `within_widget`
macro_rules! forward_mouse_events {
    ($widget_type:ty, $widget_var:ident, $vec_of_child_widgets:ident) => {
        for &event_type in crate::ui::context::MOUSE_EVENTS {
            let handler: crate::ui::context::Handler = Box::new(|obj, uictx, evt| {
                let _self = obj.downcast_mut::<$widget_type>().unwrap();
                use crate::ui::context::Handled::*;

                for w in _self.$vec_of_child_widgets.iter_mut() {
                    if within_widget(&evt.point, &w.rect()) {
                        if let Some(emitter) = w.as_emit_event() {
                            emitter.emit(evt, uictx)?;
                            return Ok(Handled);
                        }
                    }
                }

                Ok(NotHandled)
            });
            $widget_var.on(event_type, handler).unwrap();
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_enum_iter() {
        let all: Vec<EventType> = EventType::into_enum_iter().collect();
        assert_eq!(all.len(), EventType::VARIANT_COUNT);
        assert!(all.contains(&EventType::Click));
    }
}
