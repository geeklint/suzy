use drying_paint::WatchedMeta;

macro_rules! gl_object {
    ($name:ident, $create:ident, $delete:ident, $count:expr) => {
        #[derive(Debug, PartialEq)]
        struct $name {
            pub(crate) ids: [u32; $count],
            pub(crate) ready: bool,
            pub(crate) gl:
                ::std::rc::Weak<crate::platform::opengl::OpenGlBindings>,
        }
        gl_object! {impl $name, $create, $delete, $count}
    };
    (pub $name:ident, $create:ident, $delete:ident, $count:expr) => {
        #[derive(Debug, PartialEq)]
        pub struct $name {
            pub(crate) ids: [u32; $count],
            pub(crate) ready: bool,
            pub(crate) gl:
                ::std::rc::Weak<crate::platform::opengl::OpenGlBindings>,
        }
        gl_object! {impl $name, $create, $delete, $count}
    };
    (impl $name:ident, $create:ident, $delete:ident, $count:expr) => {
        impl $name {
            pub fn new() -> Self {
                Self {
                    ids: [0; $count],
                    ready: false,
                    gl: ::std::rc::Weak::new(),
                }
            }

            pub fn get(&self)
                -> Option<(
                    [u32; $count],
                    ::std::rc::Rc<crate::platform::opengl::OpenGlBindings>,
                )>
            {
                self.ids.and_then(|ids| {
                    self.gl.upgrade().map(|gl| (ids, gl))
                })
            }

            pub fn mark_ready(&mut self) {
                self.ready = true;
            }

            pub fn check_ready(
                &mut self,
                gl: &::std::rc::Rc<crate::platform::opengl::OpenGlBindings>,
            ) -> bool {
                let weak_gl = ::std::rc::Rc::downgrade(gl);
                if !self.gl.ptr_eq(weak_gl) {
                    self.drop();
                    unsafe {
                        gl.$create($count, self.ids.as_mut_ptr());
                    }
                    self.ready = false;
                    self.gl = weak_gl;
                }
                self.ready
            }
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                (self.ids == other.ids) && self.gl.ptr_eq(other.gl)
            }
        }
        impl Drop for $name {
            fn drop(&mut self) {
                // if we can't get the gl bindings here, it's probably
                // because the context went away, in which case
                // it's ok to "leak" the resource, it's already cleaned
                // up by the context going away
                if let Some(gl) = self.gl.upgrade() {
                    unsafe {
                        gl.$delete($count, self.ids.as_mut_ptr());
                    }
                }
            }
        }
    };
}
