extern crate glutin;
extern crate gl;

use std::time::{Duration};
use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};

use render::{IEngine, EngineFeatures, EngineError};
use render::{IWindow, WindowError};
use render::{ISurfaceHandler};


use self::glutin::GlContext;
use render::gl::lowlevel::LowLevel;

pub struct GLWindowImpl
{
    events_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    ll: LowLevel,
}

impl GLWindowImpl {
    fn new<T: Into<String>>(width: u32, height: u32, title: T) -> Result<GLWindowImpl, EngineError> {
        let events_loop = glutin::EventsLoop::new();
        let window_builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(width, height);
        let context_builder = glutin::ContextBuilder::new()
            .with_vsync(true);

        match glutin::GlWindow::new(window_builder, context_builder, &events_loop) {
            Err(glutin::CreationError::OsError(str)) => Err(EngineError::OsError(str)),
            Err(glutin::CreationError::RobustnessNotSupported) => Err(EngineError::FeatureNotSupported(EngineFeatures::Robustness)),
            Err(glutin::CreationError::OpenGlVersionNotSupported) => Err(EngineError::VersionNotSupported),
            Err(glutin::CreationError::NoAvailablePixelFormat) => Err(EngineError::NoAvailableFormat),
            Err(_) => Err(EngineError::Unknown),
            Ok(win) => Ok(GLWindowImpl {
                events_loop: events_loop,
                window: win,
                ll: LowLevel::new()
            }),
        }
    }

    fn release(&mut self) {
        println!("closing a window impl");
        self.ll.release();
        self.window.hide();
    }

    fn make_current(&mut self) -> Result<(), WindowError> {
        match unsafe { self.window.make_current() } {
            Err(glutin::ContextError::IoError(ioe)) => Err(WindowError::IoError(ioe)),
            Err(glutin::ContextError::ContextLost) => Err(WindowError::ContextLost),
            //Err(_) => WindowError::Unknown,
            Ok(_) => Ok(())
        }
    }

    fn init_gl_functions(&mut self) -> Result<(), WindowError> {
        match self.make_current() {
            Err(e) => Err(e),
            Ok(_) => {
                gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
                Ok(())
            }
        }
    }
}


pub struct Window {
    imp: Rc<RefCell<Option<GLWindowImpl>>>,
    surface_handler: Option<Rc<RefCell<ISurfaceHandler>>>,
    trigger_surface_ready: bool,
}

impl Window {
    pub fn render_process<F: FnMut(&mut LowLevel)>(&mut self, mut fun: F) -> Result<(), WindowError> {
        if let Some(ref mut win) = *self.imp.borrow_mut() {
            fun(&mut win.ll);
            Ok(())
        } else {
            Err(WindowError::ContextLost)
        }
    }
}

impl IWindow for Window {
    fn is_closed(&self) -> bool {
        self.imp.borrow().is_none()
    }

    fn close(&mut self) {
        if let Some(ref mut win) = *self.imp.borrow_mut() {
            win.release();
        }
        *self.imp.borrow_mut() = None;
    }

    fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        if let Some(ref mut win) = *self.imp.borrow_mut() {
            win.window.set_title(title);
            Ok(())
        } else {
            Err(WindowError::ContextLost)
        }
    }

    fn set_surface_handler<H: ISurfaceHandler>(&mut self, handler: H) {
        self.surface_handler = Some(Rc::new(RefCell::new(handler)));
    }

    fn handle_message(&mut self, timeout: Option<Duration>) -> bool {
        assert!(timeout.is_none());

        // hack to emulate create events
        if self.trigger_surface_ready && self.imp.borrow().is_some() && self.surface_handler.is_some() {
            if let Some(ref mut handler) = self.surface_handler.clone() {
                handler.borrow_mut().on_ready(self);
                self.trigger_surface_ready = false;
            }
        }

        let mut event_list = Vec::new();

        if let Some(ref mut win) = *self.imp.borrow_mut() {
            let my_window_id = win.window.id();
            win.events_loop.poll_events(|event| {
                if let glutin::Event::WindowEvent { event, window_id } = event {
                    assert_eq! (window_id, my_window_id);
                    event_list.push(event);
                }
            });
        }

        for event in event_list.into_iter() {
            match event {
                glutin::WindowEvent::Closed => {
                    if let Some(ref mut handler) = self.surface_handler.clone() {
                        handler.borrow_mut().on_lost(self);
                    }
                    self.close();
                }
                _ => (),
            }
        }


        !self.is_closed()
    }

    fn render_start(&mut self) -> Result<(), WindowError> {
        if let Some(ref mut win) = *self.imp.borrow_mut() {
            match unsafe { win.window.make_current() } {
                Err(glutin::ContextError::IoError(ioe)) => Err(WindowError::IoError(ioe)),
                Err(glutin::ContextError::ContextLost) => Err(WindowError::ContextLost),
                //Err(_) => WindowError::Unknown,
                Ok(_) => Ok(())
            }
        } else {
            Err(WindowError::ContextLost)
        }
    }

    fn render_end(&mut self) -> Result<(), WindowError> {
        if let Some(ref mut win) = *self.imp.borrow_mut() {
            match win.window.swap_buffers() {
                Err(glutin::ContextError::IoError(ioe)) => Err(WindowError::IoError(ioe)),
                Err(glutin::ContextError::ContextLost) => Err(WindowError::ContextLost),
                //Err(_) => WindowError::Unknown,
                Ok(_) => Ok(()),
            }
        } else {
            Err(WindowError::ContextLost)
        }
    }
}


pub struct GLEngineImpl {
    is_gl_initialized: bool,
    windows: Vec<Weak<RefCell<Option<GLWindowImpl>>>>,
}

impl GLEngineImpl {
    fn new() -> GLEngineImpl {
        GLEngineImpl {
            is_gl_initialized: false,
            windows: vec!(),
        }
    }

    fn remove_closed_windows(&mut self) {
        self.windows.retain(|weak_win| {
            if let Some(rc_win) = weak_win.upgrade() {
                println!("can remove: {}", rc_win.borrow().is_none());
                rc_win.borrow().is_none()
            } else {
                false
            }
        });
    }

    fn close_all_windows(&mut self) {
        for win in self.windows.iter_mut() {
            if let Some(rc_win) = win.upgrade() {
                if let Some(ref mut win) = *rc_win.borrow_mut() {
                    win.release();
                }
                *rc_win.borrow_mut() = None;
            }
        }
        self.remove_closed_windows();
    }
}

impl Drop for GLEngineImpl {
    fn drop(&mut self) {
        self.close_all_windows();
    }
}


pub struct Engine(Rc<RefCell<GLEngineImpl>>);

impl Deref for Engine {
    type Target = Rc<RefCell<GLEngineImpl>>;

    fn deref(&self) -> &Rc<RefCell<GLEngineImpl>> {
        &self.0
    }
}

impl DerefMut for Engine {
    fn deref_mut(&mut self) -> &mut Rc<RefCell<GLEngineImpl>> {
        &mut self.0
    }
}

impl IEngine for Engine {
    fn create_window<T: Into<String>>(&mut self, width: u32, height: u32, title: T) -> Result<Window, EngineError> {
        self.borrow_mut().remove_closed_windows();

        match GLWindowImpl::new(width, height, title) {
            Err(e) => Err(e),
            Ok(mut window) =>
                match if self.borrow().is_gl_initialized { window.make_current() } else { window.init_gl_functions() } {
                    Err(e) => Err(EngineError::WindowCreation(e)),
                    Ok(_) => {
                        let rc_window = Rc::new(RefCell::new(Some(window)));
                        self.borrow_mut().windows.push(Rc::downgrade(&rc_window));
                        Ok(Window {
                            imp: rc_window,
                            surface_handler: None,
                            trigger_surface_ready: true,
                        })
                    }
                }
        }
    }

    fn close_all_windows(&mut self) {
        self.borrow_mut().close_all_windows();
    }
}


pub fn create_engine() -> Result<Engine, EngineError> {
    Ok(Engine(Rc::new(RefCell::new(GLEngineImpl::new()))))
}
