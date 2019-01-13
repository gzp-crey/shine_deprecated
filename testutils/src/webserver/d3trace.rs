use actix_web::{error, Error as ActixWebError, HttpRequest, HttpResponse};
use base64;
use bytes::{BufMut, BytesMut};
use log::info;
use std::mem;
use serde_json;
use shine_gltf::{
    accessor, attribute_map, buffer, Accessor, Buffer, GetMut, Index, Mesh, Node, Primitive, Root, Scene,
};
use webserver::appcontext::AppContext;

pub trait IntoD3Data {
    fn trace(&self, tr: &mut D3Trace);
}

/// Index of an added mesh to instantiate
#[derive(Clone, Debug, PartialEq)]
pub struct MeshId(Index<Mesh>);

/// Location of a mesh instance
pub enum D3Location {
    Identity,
    Matrix([f32; 16]),
    Decomposed {
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
}

/// Trace 3D geometry object through the web service
pub struct D3Trace {
    root: Root,
}

impl D3Trace {
    pub fn new() -> D3Trace {
        let mut root = Root::default();
        let scene_id = root.add_scene(Scene::default());
        root.scene = Some(scene_id);

        D3Trace { root }
    }

    fn create_geometry<V, I>(&mut self, positions: V, indices: I) -> Primitive
    where
        V: IntoIterator<Item = (f32, f32, f32)>,
        I: IntoIterator<Item = u32>,
    {
        let mut data = BytesMut::new();

        let position_byte_stride = 3 * mem::size_of::<f32>();
        for (x, y, z) in positions.into_iter() {
            if data.remaining_mut() < position_byte_stride {
                // reserve more
                data.reserve(position_byte_stride * 1024);
            }

            data.put_f32_le(x);
            data.put_f32_le(y);
            data.put_f32_le(z);
        }
        let position_byte_count = data.len();
        let position_byte_offset = 0;        
        let position_count = position_byte_count / position_byte_stride;

        let index_byte_stride = mem::size_of::<u32>();
        for i in indices.into_iter() {
            if data.remaining_mut() < index_byte_stride {
                // reserve more
                data.reserve(index_byte_stride * 1024);
            }

            data.put_u32_le(i);
        }
        let index_byte_count = data.len() - position_byte_count;
        let index_byte_offset = position_byte_count;
        let index_count = index_byte_count / index_byte_stride;

        let encoded_data = base64::encode(&data);

        let buffer_id = {
            let buffer = Buffer {
                byte_length: data.len() as u32,
                uri: Some(format!("data:application/octet-stream;base64,{}", encoded_data)),
                ..Default::default()
            };
            self.root.add_buffer(buffer)
        };

        let position_accessor_id = {
            let view_id = {
                let view = buffer::View {
                    byte_length: position_byte_count as u32,
                    byte_offset: Some(position_byte_offset as u32),
                    byte_stride: Some(buffer::ByteStride(position_byte_stride as u32)),
                    ..buffer::View::with_buffer(buffer_id.clone())
                };
                self.root.add_buffer_view(view)
            };

            let accessor = Accessor {
                count: position_count as u32,
                ..Accessor::with_view(view_id, accessor::Type::Vec3, accessor::ComponentType::F32, false)
            };
            self.root.add_accessor(accessor)
        };

        let index_accessor_id = {
            let view_id = {
                let buffer_view = buffer::View {
                    byte_length: index_byte_count as u32,
                    byte_offset: Some(index_byte_offset as u32),
                    byte_stride: Some(buffer::ByteStride(index_byte_stride as u32)),
                    ..buffer::View::with_buffer(buffer_id.clone())
                };
                self.root.add_buffer_view(buffer_view)
            };

            let accessor = Accessor {
                count: index_count as u32,
                ..Accessor::with_view(view_id, accessor::Type::Scalar, accessor::ComponentType::U32, false)
            };
            self.root.add_accessor(accessor)
        };

        Primitive {
            attributes: attribute_map![ Positions => position_accessor_id ],
            indices: Some(index_accessor_id),
            ..Primitive::default()
        }
    }

    pub fn add_indexed_mesh<V, I>(&mut self, positions: V, indices: I) -> MeshId
    where
        V: IntoIterator<Item = (f32, f32, f32)>,
        I: IntoIterator<Item = u32>,
    {
        let geometry = self.create_geometry(positions, indices);

        let mesh_id = {
            let mesh = Mesh {
                primitives: vec![geometry],
                ..Default::default()
            };
            self.root.add_mesh(mesh)
        };

        MeshId(mesh_id)
    }

    pub fn add_instance(&mut self, mesh: MeshId, location: D3Location) {
        let node_id = {
            let node = Node {
                mesh: Some(mesh.0),
                ..Default::default()
            };
            self.root.add_node(node)
        };

        {
            let scene = self.root.scene.as_ref().unwrap().clone();
            let scene = self.root.get_mut(&scene).unwrap();
            scene.nodes.push(node_id);
        }

        info!("{}", self.root.to_string_pretty().unwrap());
    }

    pub fn add_indexed_mesh_instance<V, I>(&mut self, positions: V, indices: I, location: D3Location) -> MeshId
    where
        V: IntoIterator<Item = (f32, f32, f32)>,
        I: IntoIterator<Item = u32>,
    {
        let id = self.add_indexed_mesh(positions, indices);
        self.add_instance(id.clone(), location);
        id
    }

    pub fn to_data(self) -> String {
        serde_json::to_string(&self.root).unwrap()
    }
}

impl Default for D3Trace {
    fn default() -> D3Trace {
        D3Trace::new()
    }
}

pub fn handle_d3data_request(req: &HttpRequest<AppContext>) -> Result<HttpResponse, ActixWebError> {
    let state = req.state();

    let id: usize = match req.query().get("id") {
        Some(id) => id
            .parse()
            .map_err(|_| error::ErrorBadRequest(format!("Invalid id: {}", id)))?,
        None => 0,
    };

    let data = {
        info!("Getting d3data for {}", id);
        let mut d3datas = state.d3datas.lock().unwrap();
        if id >= d3datas.len() {
            "".into()
        } else {
            d3datas[id].clone()
        }
    };

    Ok(HttpResponse::Ok().content_type("application/json").body(data))
}

pub fn handle_d3datas_request(req: &HttpRequest<AppContext>) -> Result<HttpResponse, ActixWebError> {
    let state = req.state();

    info!("Getting all d3datas");
    let data = {
        let d3datas = state.d3datas.lock().unwrap();
        d3datas.join(",")
    };
    let data = format!("[{}]", data);
    Ok(HttpResponse::Ok().content_type("application/json").body(data))
}

pub fn handle_d3view_request(req: &HttpRequest<AppContext>) -> Result<HttpResponse, ActixWebError> {
    let state = req.state();

    let all_data = {
        let img = state.d3datas.lock().unwrap();
        serde_json::to_string(&*img).unwrap()
    };

    let mut ctx = tera::Context::new();
    ctx.insert("model_list", &all_data);

    let body = state.template.render("d3view.html", &ctx).map_err(|e| {
        println!("Template error: {}", e);
        error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
