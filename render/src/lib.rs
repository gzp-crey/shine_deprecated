#![feature(macro_reexport)]

extern crate shine_render_core;

pub use shine_render_core::*;

mod backend_impl {
    extern crate shine_render_gl;

    pub use self::shine_render_gl::*;
    //pub use self::shine_render_gl::VertexDeclaration;
    //pub use self::shine_render_gl::ShaderDeclaration;

    pub use self::shine_render_gl::PlatformEngine;
    pub use self::shine_render_gl::PlatformWindowSettings;
    pub use self::shine_render_gl::PlatformWindowBuilder;

    pub use self::shine_render_gl::VertexBufferHandle;
    pub use self::shine_render_gl::IndexBufferHandle;
    pub use self::shine_render_gl::Texture2DHandle;
    pub use self::shine_render_gl::ShaderProgramHandle;
}

pub use backend_impl::*;

pub type PlatformBackend = <PlatformEngine as Engine>::Backend;
pub type PlatformCommandQueue = <PlatformEngine as Engine>::CommandQueue;
