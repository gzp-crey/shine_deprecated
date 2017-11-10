use std::mem::transmute;
use std::rc::Rc;
use std::cell::RefCell;

use world::*;
use dragorust_engine::render::*;

#[derive(Copy, Clone, Debug)]
#[derive(VertexDeclaration)]
struct VxPos {
    position: Float32x3,
}

#[derive(Copy, Clone, Debug)]
#[derive(VertexDeclaration)]
struct VxColorTex {
    color: Float32x3,
    tex_coord: Float32x2,
}


#[derive(ShaderDeclaration)]
#[vert_path = "common.glsl"]
#[vert_src = "
    attribute vec3 vPosition;
    void main()
    {
        gl_Position = vec4(vPosition, 1.0);
    }
"]
#[frag_path = "common.glsl"]
#[frag_src = "
    void main()
    {
        gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    }
"]
struct ShSimple {}


impl ShaderDeclaration for ShSimple {
    type Attribute = ShSimpleAttribute;
    type Uniform = ShSimpleUniform;

    fn map_sources<F: FnMut((ShaderType, &str)) -> bool>(mut f: F) -> bool {
        let sh_source = [
            (ShaderType::VertexShader,
             r#"
attribute vec3 vPosition;
void main()
{
    gl_Position = vec4(vPosition, 1.0);
}
"#),
            (ShaderType::FragmentShader,
             r#"
void main()
{
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}"#)];

        for src in sh_source.iter() {
            if !f(*src) {
                return false
            }
        }

        true
    }
}

#[derive(Copy, Clone, Debug)]
#[derive(PrimitiveEnum)]
#[repr(usize)]
enum ShSimpleAttribute {
    #[name = "vPosition"]
    Position,

    #[name = "vColor"]
    Color,

    #[name = "vTexCoord"]
    TexCoord,
}

#[derive(Copy, Clone, Debug)]
#[derive(PrimitiveEnum)]
#[repr(usize)]
enum ShSimpleUniform {
    #[name = "uProjModel"]
    ProjModel,

    #[name = "uColor"]
    Color,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Passes {
    Present,
    //Shadow,
    //Debug,
}

impl PassKey for Passes {}


/// Structure to handle view dependent data
pub struct SimpleView {
    world: Rc<RefCell<World>>,
    render: RenderManager<Passes>,

    shader: ShaderProgram<ShSimple>,

    vertex_buffer1: VertexBuffer<VxPos>,
    vertex_buffer2: VertexBuffer<VxColorTex>,

    t: f32,
}

impl SimpleView {
    pub fn new(world: Rc<RefCell<World>>) -> SimpleView {
        SimpleView {
            world: world,
            render: RenderManager::new(),
            shader: ShaderProgram::new(),
            vertex_buffer1: VertexBuffer::new(),
            vertex_buffer2: VertexBuffer::new(),
            t: 0f32,
        }
    }
}

impl View for SimpleView {
    fn on_surface_ready(&mut self, window: &mut Window) {
        // create some shaders


        // create some geometry
        let pos = [
            VxPos { position: f32x3!(1f32, 0f32, 0f32) },
            VxPos { position: f32x3!(1f32, 1f32, 0f32) },
            VxPos { position: f32x3!(0f32, 1f32, 0f32) }
        ];

        // create some geometry
        let color_tex = [
            VxColorTex { color: f32x3!(1f32, 0f32, 0f32), tex_coord: f32x2!(1, 0) },
            VxColorTex { color: f32x3!(1f32, 1f32, 0f32), tex_coord: f32x2!(1, 1) },
            VxColorTex { color: f32x3!(0f32, 1f32, 0f32), tex_coord: f32x2!(0, 0) }
        ];

        // upload data
        //self.shader.set_sources(&mut self.render, sh_source.iter());
        self.shader.compile(&mut self.render);
        self.vertex_buffer1.set_transient(&mut self.render, &pos);
        self.vertex_buffer2.set_transient(&mut self.render, &color_tex.to_vec());

        // submit commands
        self.render.submit(window);
    }

    fn on_surface_lost(&mut self, window: &mut Window) {
        self.vertex_buffer1.release(&mut self.render);
        self.vertex_buffer2.release(&mut self.render);
        self.shader.release(&mut self.render);
        self.render.submit(window);
    }

    fn on_surface_changed(&mut self, _window: &mut Window) {
        // nop
    }

    fn on_update(&mut self) {
        self.t += 0.005f32;
        if self.t > 1f32 {
            self.t = 0f32;
        }
    }

    fn on_render(&mut self, window: &mut Window) {
        {
            let mut p0 = self.render.get_pass(Passes::Present);
            p0.config_mut().set_clear_color(f32x3!(self.t, self.world.borrow().get_t(), 0.));
            p0.config_mut().set_fullscreen();

            let v1 = &self.vertex_buffer1;
            let v2 = &self.vertex_buffer2;

            self.shader.draw(&mut *p0,
                             |id| match id {
                                 ShSimpleAttribute::Position => v1.get_attribute(VxPosAttribute::Position),
                                 ShSimpleAttribute::Color => v2.get_attribute(VxColorTexAttribute::Color),
                                 ShSimpleAttribute::TexCoord => v2.get_attribute(VxColorTexAttribute::TexCoord),
                             },
                             Primitive::Triangle, 0, 3
            );
        }

        {
            //let mut p1 = self.render.get_pass(Passes::Shadow);
        }

        self.render.submit(window);
    }

    fn on_key(&mut self, _window: &mut Window,
              _scan_code: ScanCode, _virtual_key: Option<VirtualKeyCode>, _is_down: bool) {}
}