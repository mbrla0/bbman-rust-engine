#[derive(Copy, Clone)]
#[allow(non_snake_case)]
struct Vertex{
	bbm_Position: [f32; 3],
	bbm_TexCoord: [f32; 2],
	bbm_Normal:   [f32; 3]
}
implement_vertex!(Vertex, bbm_Position, bbm_TexCoord, bbm_Normal);

use glium::Texture2d;
pub trait TextureProvider{
	fn get_texture(&self) -> &Texture2d;
}
impl TextureProvider for Texture2d{
	fn get_texture(&self) -> &Texture2d { self }
}

use image::ImageError;
use glium::texture::TextureCreationError;
#[derive(Debug)]
pub enum TextureError{
	Image(ImageError),
	TextureCreation(TextureCreationError)
}

use glium::texture::{RawImage2d, ClientFormat};
use glium::backend::Facade;
use super::grid::Grid;
pub struct Texture(Texture2d);
impl Texture{
	pub fn open<F: Facade>(facade: &F, path: &str) -> Result<Texture, TextureError>{
		use image;
		use std::borrow::Cow;
		match image::open(path){
			Ok(img) => {
				let rgba = img.to_rgba();
				let data = RawImage2d{
					width:  rgba.width(),
					height: rgba.height(),
					format: ClientFormat::U8U8U8U8,
					data:   Cow::Owned(rgba.into_raw())
				};

				match Texture2d::new(facade, data){
					Ok(texture) => Ok(Texture(texture)),
					Err(what) => {
						error!("Could not upload texture to OpenGL: {:?}", what);
						Err(TextureError::TextureCreation(what))
					}
				}
			}
			Err(what) => {
				error!(r#"Could not open image at "{}": {:?}"#, path, what);
				Err(TextureError::Image(what))
			}
		}
	}

	pub fn open_from_memory<F: Facade>(facade: &F, source: &[u8]) -> Result<Texture, TextureError>{
		use image;
		use std::borrow::Cow;
		match image::load_from_memory(source){
			Ok(img) => {
				let rgba = img.to_rgba();
				let data = RawImage2d{
					width:  rgba.width(),
					height: rgba.height(),
					format: ClientFormat::U8U8U8U8,
					data:   Cow::Owned(rgba.into_raw())
				};

				match Texture2d::new(facade, data){
					Ok(texture) => Ok(Texture(texture)),
					Err(what) => {
						error!("Could not upload texture to OpenGL: {:?}", what);
						Err(TextureError::TextureCreation(what))
					}
				}
			}
			Err(what) => {
				error!("Could not open image from memory: {:?}", what);
				Err(TextureError::Image(what))
			}
		}
	}

	pub fn sprite_sheet<F: Facade>(self, facade: &F, sprite_width: usize, sprite_height: usize) -> Result<Grid<Texture>, TextureError>{
		// Get the grid's dimensions in sprites
		let dimensions = (
			(self.0.width()  as f64 / sprite_width  as f64).floor() as usize,
			(self.0.height() as f64 / sprite_height as f64).floor() as usize
		);

		// Read the texture's raw data
		let texture = self.0.read::<RawImage2d<u8>>();

		// Read the sheet's data into sectors
		use std::mem;
		let mut grid = Grid::new(sprite_width, sprite_height, 1, dimensions.0, dimensions.1, 1);
		for y in 0..dimensions.1{
			for x in 0..dimensions.0{
				// Get pixel data for the current sprite
				let mut sprite = Vec::with_capacity(sprite_width * sprite_height);
				for b in 0..sprite_height{
					for a in 0..sprite_width{
						let bpp = texture.format.get_size();
						for e in 0..bpp{
							let offset_y = y * sprite_height + sprite_height - 1 - b;
							let offset_x = x * sprite_width  + a;

							// Calculate the offset
							let offset = offset_y * texture.width as usize + offset_x;

							sprite.push(texture.data[offset * bpp + e]);
						}
					}
				}

				use std::borrow::Cow;
				grid.push(y, 0, match Texture2d::new(facade, RawImage2d{
					format: texture.format,
					width:  sprite_width  as u32,
					height: sprite_height as u32,
					data:   Cow::Owned(sprite)
				}){
					Ok(sprite) => Texture(sprite),
					Err(what) => {
						error!("Could not create texture for sprite at ({}, {}): {:?}", x, y, what);
						return Err(TextureError::TextureCreation(what))
					}
				});
			}
		}

		// Return the newly created grid
		Ok(grid)
	}
}
impl TextureProvider for Texture{
	fn get_texture(&self) -> &Texture2d{ &self.0 }
}

use super::DeltaTimer;
pub struct Animation{
	pub frames: Vec<Texture2d>,

	pub timing:      usize, /* The time each frame will be displayed for, in milliseconds */
 	pub loop_offset: usize, /* Number of frames that will be skipped every iteration of the loop */

	current_frame:  usize,
	timer: DeltaTimer
}
impl Animation{
	fn update(&mut self){
		let delta_millis = self.timer.delta_millis();

		// Check if any frames have passed
		if delta_millis >= self.timing as u64{
			// Increment as many frames as needed
			self.current_frame += (self.timing as f64 / delta_millis as f64).floor() as usize;

			// Loop animation if necesarry
			while self.current_frame >= self.frames.len(){ self.current_frame -= self.frames.len() + self.loop_offset }
		}
	}
}
impl TextureProvider for Animation{
	fn get_texture(&self) -> &Texture2d{
		&self.frames[self.current_frame]
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

use glium::{Surface, VertexBuffer, Program, DrawError};
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
					error!("Could not register the required Rectangle Vertex Buffer: {:?}", what);
					return None
				}
			},
			texture_shader: match Program::from_source(facade, RENDERER2D_TEXTURE_VERTEX_SHADER, RENDERER2D_TEXTURE_FRAGMENT_SHADER, None){
				Ok(program) => program,
				Err(what) => {
					error!("Could not compile and link texture shader program: {:?}", what);
					return None
				}
			},
			color_shader: match Program::from_source(facade, RENDERER2D_COLOR_VERTEX_SHADER, RENDERER2D_COLOR_FRAGMENT_SHADER, None){
				Ok(program) => program,
				Err(what) => {
					error!("Could not compile and link color shader program: {:?}", what);
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

#[cfg(test)]
mod tests{
	use super::Texture;

	#[test]
	fn spritesheet(){
		// Setup logger
		let _ = ::setup_logger();

		// Setup headless context
		use glium::DisplayBuild;
		use glium::glutin::WindowBuilder;
		let display = WindowBuilder::new()
			.with_dimensions(1280, 720)
			.with_title("Automated test: grid::spritesheet()")
			.build_glium().unwrap();

		// Load and create a new image and turn it into a spritesheet
		let spritesheet = Texture::open(&display, "test/sheet.png").unwrap().sprite_sheet(&display, 16, 16).unwrap();

		assert_eq!(spritesheet.elements.len(),       1);
		assert_eq!(spritesheet.elements[0].len(),    2);
		assert_eq!(spritesheet.elements[0][0].len(), 2);
		assert!(spritesheet.in_range(1, 1, 0));
		assert_eq!(spritesheet.flatten().len(), 4);
	}
}
