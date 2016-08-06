#[derive(Copy, Clone)]
#[allow(non_snake_case)]
struct Vertex{
	bbm_Position: [f32; 3],
	bbm_TexCoord: [f32; 2],
	bbm_Normal:   [f32; 3]
}
implement_vertex!(Vertex, bbm_Position, bbm_TexCoord, bbm_Normal);

use super::glium::Texture2d;
pub trait TextureProvider{
	fn get_texture(&self) -> &Texture2d;
}
impl TextureProvider for Texture2d{
	fn get_texture(&self) -> &Texture2d { self }
}

use glium::texture::{RawImage2d, ClientFormat};
use glium::backend::Facade;
pub enum Texture{
	None,
	Loaded(Texture2d)
}
impl Texture{
	pub fn open<F: Facade>(facade: &F, path: &str) -> Option<Texture>{
		use super::image;
		use std::borrow::Cow;
		if let Ok(img) = image::open(path){
			let rgba = img.to_rgba();
			let data = RawImage2d{
				width:  rgba.width(),
				height: rgba.height(),
				format: ClientFormat::U8U8U8U8,
				data:   Cow::Owned(rgba.into_raw())
			};

			match Texture2d::new(facade, data){
				Ok(texture) => Some(Texture::Loaded(texture)),
				Err(what) => {
					println!("[Error][Texture] Could not upload texture to OpenGL: {:?}", what);
					None
				}
			}
		} else { None }
	}

	pub fn open_from_memory<F: Facade>(facade: &F, source: &[u8]) -> Option<Texture>{
		use super::image;
		use std::borrow::Cow;
		if let Ok(img) = image::load_from_memory(source){
			let rgba = img.to_rgba();
			let data = RawImage2d{
				width:  rgba.width(),
				height: rgba.height(),
				format: ClientFormat::U8U8U8U8,
				data:   Cow::Owned(rgba.into_raw())
			};

			match Texture2d::new(facade, data){
				Ok(texture) => Some(Texture::Loaded(texture)),
				Err(what) => {
					println!("[Error][Texture] Could not upload texture to OpenGL: {:?}", what);
					None
				}
			}
		} else { None }
	}
}
impl Texture{
	pub fn is_loaded(&self) -> bool{
		match self{
			&Texture::Loaded(_) => true,
			_ => false
		}
	}

	pub fn is_none(&self) -> bool{
		match self{
			&Texture::None => true,
			_ => false
		}
	}
}
impl TextureProvider for Texture{
	fn get_texture(&self) -> &Texture2d{
		match *self{
			Texture::Loaded(ref texture) => &texture,
			_ => panic!("Texture is not loaded")
		}
	}
}

static RENDERER2D_TEXTURE_VERTEX_SHADER: &'static str = "
	#version 140

	in vec3 bbm_Position;
	in vec2 bbm_TexCoord;
	in vec3 bbm_Normal;

	out vec2 texcoord;

	uniform mat4 bbm_Matrix;
	void main(){ texcoord = bbm_TexCoord; gl_Position = bbm_Matrix * vec4(bbm_Position, 1); }
";
static RENDERER2D_COLOR_VERTEX_SHADER: &'static str = "
	#version 140

	in vec3 bbm_Position;
	in vec2 bbm_TexCoord;
	in vec3 bbm_Normal;

	out vec2 texcoord;

	uniform mat4 bbm_Matrix;
	void main(){ gl_Position = bbm_Matrix * vec4(bbm_Position, 1); }
";
static RENDERER2D_TEXTURE_FRAGMENT_SHADER: &'static str = "
	#version 140

	in vec2 texcoord;
	out vec4 color;

	uniform vec4 bbm_Color;
	uniform sampler2D bbm_Texture;
	void main(){ color = texture(bbm_Texture, texcoord) * bbm_Color; }
";
static RENDERER2D_COLOR_FRAGMENT_SHADER: &'static str = "
	#version 140

	in vec2 texcoord;
	out vec4 color;

	uniform vec4 bbm_Color;
	void main(){ color = bbm_Color; }
";


use super::glium::{Surface, VertexBuffer, Program, DrawError};
use super::Camera;
pub struct Renderer2d{
	camera: Camera,
	rectangle: VertexBuffer<Vertex>,
	texture_shader: Program,
	color_shader:   Program
}
impl Renderer2d{
	pub fn new<F: Facade>(facade: &F, width: f32, height: f32) -> Option<Renderer2d>{
		use cgmath;
		Some(Renderer2d{
			camera: Camera::new(cgmath::ortho(0.0, width, height, 0.0, 1.0, -1.0)),
			rectangle: match VertexBuffer::new(facade, &[
				Vertex{ bbm_Position: [0.0, 0.0, 0.0], bbm_TexCoord: [0.0, 0.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [1.0, 0.0, 0.0], bbm_TexCoord: [1.0, 0.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [1.0, 1.0, 0.0], bbm_TexCoord: [1.0, 1.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [0.0, 1.0, 0.0], bbm_TexCoord: [0.0, 1.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [0.0, 0.0, 0.0], bbm_TexCoord: [0.0, 0.0], bbm_Normal: [0.0, 0.0, 0.0] }
			]){
				Ok(vb) => vb,
				Err(what) => {
					println!("[Error][Renderer2d] Could not register the required Rectangle Vertex Buffer: {:?}", what);
					return None
				}
			},
			texture_shader: match Program::from_source(facade, RENDERER2D_TEXTURE_VERTEX_SHADER, RENDERER2D_TEXTURE_FRAGMENT_SHADER, None){
				Ok(program) => program,
				Err(what) => {
					println!("[Error][Renderer2d] Could not compile and link texture shader program: {:?}", what);
					return None
				}
			},
			color_shader: match Program::from_source(facade, RENDERER2D_COLOR_VERTEX_SHADER, RENDERER2D_COLOR_FRAGMENT_SHADER, None){
				Ok(program) => program,
				Err(what) => {
					println!("[Error][Renderer2d] Could not compile and link color shader program: {:?}", what);
					return None
				}
			}
		})
	}

	pub fn resize(&mut self, width: f32, height: f32){
		use cgmath;
		self.camera.replace_projection(cgmath::ortho(0.0, width, height, 0.0, 1.0, -1.0));
	}

	pub fn rectangle<S: Surface>(&mut self, target: &mut S, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32)) -> Result<(), DrawError>{
		use glium::index::{NoIndices, PrimitiveType};
		self.camera.push();

		self.camera.translate(x, y, 0.0);
		self.camera.scale(width, height, 1.0);
		let matrix: [[f32; 4]; 4] = self.camera.get_matrix().into();

		self.camera.pop();

		let uniform = uniform!{
			bbm_Matrix: matrix,
			bbm_Color:  [color.0, color.1, color.2, color.3]
		};

		// Draw the quad
		target.draw(&self.rectangle, NoIndices(PrimitiveType::TriangleStrip), &self.color_shader, &uniform, &Default::default())
	}

	pub fn shaded_sprite<S: Surface, T: TextureProvider>(&mut self, target: &mut S, x: f32, y: f32, width: f32, height: f32, texture: &T, color: (f32, f32, f32, f32)) -> Result<(), DrawError>{
		use glium::index::{NoIndices, PrimitiveType};
		self.camera.push();

		self.camera.translate(x, y, 0.0);
		self.camera.scale(width, height, 1.0);
		let matrix: [[f32; 4]; 4] = self.camera.get_matrix().into();

		self.camera.pop();

		let uniform = uniform!{
			bbm_Matrix:  matrix,
			bbm_Texture: texture.get_texture(),
			bbm_Color:   [color.0, color.1, color.2, color.3]
		};
		// Draw the quad
		target.draw(&self.rectangle, NoIndices(PrimitiveType::TriangleStrip), &self.texture_shader, &uniform, &Default::default())

	}

	pub fn sprite<S: Surface, T: TextureProvider>(&mut self, target: &mut S, x: f32, y: f32, width: f32, height: f32, texture: &T) -> Result<(), DrawError>{
		self.shaded_sprite(target, x, y, width, height, texture, (1.0, 1.0, 1.0, 1.0))
	}
}
