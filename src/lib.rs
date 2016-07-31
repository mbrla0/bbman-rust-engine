#[macro_use] extern crate glium;  /* For everything OpenGL  */
#[macro_use] extern crate cgmath; /* For Matrix calculation */
#[macro_use] extern crate json;   /* For JSON parsing       */
#[macro_use] extern crate image;  /* For image decoding     */

/* Use CgMath's structures */
pub use cgmath::*;

/* Import all submodules */
pub mod grid;

pub mod physics;
pub use self::physics::Dynamic;

pub mod transform;
pub use self::transform::Camera;

pub mod world;
pub mod game;

use std::time::{Instant, Duration};
pub struct DeltaTimer(Option<Instant>);
impl DeltaTimer{
	pub fn new() -> DeltaTimer{
		DeltaTimer(None)
	}

	/* Reset the timer */
	pub fn reset(&mut self){ self.0 = None }

	pub fn duration(&mut self) -> Duration{
		let now = Instant::now();
		let delta = now.duration_since(match self.0 { Some(t) => t, None => now.clone() });

		// Save the current time
		self.0 = Some(now);
		delta
	}

	pub fn delta_millis(&mut self) -> u64{
		let duration = self.duration();

		(duration.as_secs() * 1000) + (duration.subsec_nanos() as f64 / 1_000_000.0).floor() as u64
	}

	pub fn delta_nanos(&mut self) -> u64{
		let duration = self.duration();

		(duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64
	}

	pub fn delta_seconds(&mut self) -> u64{
		self.delta().round() as u64
	}

	pub fn delta(&mut self) -> f64{
		let duration = self.duration();

		(duration.as_secs() as f64) + (duration.subsec_nanos() as f64 / 1_000_000_000.0)
	}
}
#[test]
fn delta_timer(){
	let mut timer = DeltaTimer::new();
	let _ = timer.duration();

	use std::thread;
	thread::sleep_ms(2000);

	assert_eq!(timer.delta_seconds(), 2);
}

pub trait Update{
	fn update(&mut self, delta: &f64);
}

use glium::Texture2d;
pub trait TextureProvider{
	fn get_texture(&mut self) -> &Texture2d;
}

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
	fn get_texture(&mut self) -> &Texture2d{
		self.update();
		&self.frames[self.current_frame]
	}
}

// ======================= //
// === Automated Tests === //
// ======================= //
#[test]
fn headed_opengl(){
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: headed_opengl()")
        .build_glium().unwrap();

    use glium::backend::Facade;
    let context = display.get_context();
    println!("Built context has: OpenGL {:?}, GLSL {:?}", context.get_opengl_version(), context.get_supported_glsl_version());
}

#[test]
fn render_register_quad(){
    // Setup headless context
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: render_register_quad()")
        .build_glium().unwrap();

    // Register quad
    #[derive(Copy, Clone)]
    struct Vertex{
        position:   [f32; 3],
        tex_coords: [f32; 2]
    }
    implement_vertex!(Vertex, position, tex_coords);

    use glium::VertexBuffer;
    let quad = VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 1.0] },
        Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 1.0] },
    ]).unwrap();
}

#[test]
fn shader_140(){
	static vert: &'static str = "
	    #version 140

	    in vec3 position;
	    in vec2 tex_coords;

	    out vec2 tex;

	    uniform mat4 bbm_GridMatrix;
	    void main(){
	        tex = tex_coords;
	        gl_Position = bbm_GridMatrix * vec4(position, 1.0);
	    }
	";
	static frag: &'static str = "
	    #version 140

	    in vec2 tex;
	    out vec4 color;

	    uniform sampler2D bbm_TileTexture;
	    void main(){
	        color = texture(bbm_TileTexture, tex);
	    }
	";

    // Setup headless context
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: grid_render_shader()")
        .build_glium().unwrap();


    // Compiler GRIDRENDER shader
    use glium::Program;
    let shader = Program::from_source(&display, vert, frag, None).unwrap();
}
