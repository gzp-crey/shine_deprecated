use actix_web::{error, Error as ActixWebError, HttpRequest, HttpResponse};
use std::collections::HashMap;
use svg::node::{element, Text};
use svg::{Document, Node};
use tera;
use webserver::service::AppContext;

pub trait IntoD2Image {
    fn trace(&self, tr: &mut D2Trace);
}

enum Container {
    Root(Document),
    Layer(element::Group),
}

impl Container {
    fn add_node<N: Node>(&mut self, node: N) {
        match *self {
            Container::Layer(ref mut group) => group.append(node),
            Container::Root(ref mut doc) => doc.append(node),
        }
    }
}

struct Layer {
    container: Container,
    texts: HashMap<(i32, i32), Vec<(String, String)>>,
}

impl Layer {
    fn new_root() -> Layer {
        Layer {
            container: Container::Root(Document::new()),
            texts: HashMap::new(),
        }
    }

    fn new_layer() -> Layer {
        Layer {
            container: Container::Layer(element::Group::new()),
            texts: HashMap::new(),
        }
    }

    fn add_text(&mut self, p: (f64, f64), msg: String, color: String) {
        let key = ((p.0 * 65536.) as i32, (p.1 * 65546.) as i32);
        self.texts.entry(key).or_insert(Vec::new()).push((msg, color));
    }

    fn add_node<N: Node>(&mut self, node: N) {
        self.container.add_node(node);
    }

    fn finalize(mut self) -> Container {
        if !self.texts.is_empty() {
            for (pos, texts) in self.texts.iter() {
                let p = (pos.0 as f32 / 65536., pos.1 as f32 / 65536.);
                let mut group = element::Group::new()
                    .set("preserve-size", "true")
                    .set("transform", format!("translate({},{}) scale(1)", p.0, p.1));
                for (i, text) in texts.iter().enumerate() {
                    let mut node = element::Text::new()
                        .set("x", 0)
                        .set("y", 0.05 + i as f32 * 0.05)
                        .set("font-size", "0.05")
                        .set("fill", text.1.clone());
                    node.append(Text::new(text.0.clone()));
                    group.append(node);
                }
                self.container.add_node(group);
            }
        }

        self.container
    }
}

/// Trace 2D geometry object through the web service
pub struct D2Trace {
    layers: Vec<Layer>,
    scale: (f64, f64, f64, f64),
}

impl D2Trace {
    pub fn new() -> D2Trace {
        D2Trace {
            layers: vec![Layer::new_root()],
            scale: (1., 1., 0., 0.),
        }
    }

    pub fn push_layer(&mut self) {
        self.layers.push(Layer::new_layer());
    }

    pub fn pop_layer(&mut self) {
        let layer = self.layers.pop().unwrap();
        match layer.finalize() {
            Container::Layer(group) => self.add_node(group),
            _ => panic!("Poping root layer"),
        }
    }

    pub fn pop_all_layers(&mut self) {
        while self.layers.len() > 1 {
            self.pop_layer();
        }
    }

    pub fn document(mut self) -> Document {
        self.pop_all_layers();

        let layer = self.layers.pop().unwrap();
        match layer.finalize() {
            Container::Root(mut document) => {
                document.assign("width", "640");
                document.assign("viewbox", "-1 -1 2 2");
                document
            }

            _ => panic!("Poping root layer"),
        }
    }

    pub fn set_scale(&mut self, minx: f64, miny: f64, maxx: f64, maxy: f64) {
        let w = maxx - minx;
        let h = maxy - miny;
        let w = if w == 0. { 1. } else { w };
        let h = if h == 0. { 1. } else { h };

        self.scale.0 = 2. / w;
        self.scale.1 = 2. / h;
        self.scale.2 = -(minx + maxx) / w;
        self.scale.3 = -(miny + maxy) / h;
    }

    pub fn scale_position(&self, p: &(f64, f64)) -> (f64, f64) {
        (p.0 * self.scale.0 + self.scale.2, p.1 * self.scale.1 + self.scale.3)
    }

    pub fn add_point(&mut self, p: &(f64, f64), color: String) {
        let p = self.scale_position(p);
        let node = element::Line::new()
            .set("x1", p.0)
            .set("y1", p.1)
            .set("x2", p.0)
            .set("y2", p.1)
            .set("vector-effect", "non-scaling-stroke")
            .set("stroke-linecap", "round")
            .set("stroke", color)
            .set("stroke-width", "4");
        self.add_node(node);
    }

    pub fn add_line(&mut self, a: &(f64, f64), b: &(f64, f64), color: String) {
        let a = self.scale_position(a);
        let b = self.scale_position(b);
        let node = element::Line::new()
            .set("x1", a.0)
            .set("y1", a.1)
            .set("x2", b.0)
            .set("y2", b.1)
            .set("vector-effect", "non-scaling-stroke")
            .set("stroke-linecap", "round")
            .set("stroke", color)
            .set("stroke-width", "2");
        self.add_node(node);
    }

    pub fn add_text(&mut self, p: &(f64, f64), msg: String, color: String) {
        let p = self.scale_position(p);

        let layer = self.layers.last_mut().unwrap();
        layer.add_text(p, msg, color);
    }

    fn add_node<N: Node>(&mut self, node: N) {
        let layer = self.layers.last_mut().unwrap();
        layer.add_node(node);
    }
}

impl Default for D2Trace {
    fn default() -> D2Trace {
        D2Trace::new()
    }
}

crate fn d2_page(req: &HttpRequest<AppContext>) -> Result<HttpResponse, ActixWebError> {
    let state = req.state();

    let id = match req.query().get("id") {
        Some(id) => id
            .parse()
            .map_err(|_| error::ErrorBadRequest(format!("Invalid id: {}", id)))?,
        None => 0,
    };

    let (image, id, image_count) = {
        let mut img = state.d2_images.lock().unwrap();
        if img.is_empty() {
            ("<svg></svg>".into(), 0, 1)
        } else if id < img.len() {
            (img[id].clone(), id, img.len())
        } else {
            (img.last().unwrap().clone(), img.len() - 1, img.len())
        }
    };

    let last_id = image_count - 1;
    let next_id = if id + 1 <= last_id { id + 1 } else { last_id };
    let next_next_id = if id + 10 <= last_id { id + 10 } else { last_id };
    let prev_id = if id > 1 { id - 1 } else { 0 };
    let prev_prev_id = if id > 10 { id - 10 } else { 0 };

    let mut ctx = tera::Context::new();
    ctx.insert("image_id", &format!("{}", id));
    ctx.insert("image_count", &image_count);
    ctx.insert("next_image_id", &next_id);
    ctx.insert("next_next_image_id", &next_next_id);
    ctx.insert("prev_image_id", &prev_id);
    ctx.insert("prev_prev_image_id", &prev_prev_id);
    ctx.insert("last_image_id", &last_id);
    ctx.insert("svg", &image);

    let body = state.template.render("d2.html", &ctx).map_err(|e| {
        println!("Template error: {}", e);
        error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
