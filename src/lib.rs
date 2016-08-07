#[macro_use] extern crate glium;  /* For everything OpenGL   */
#[macro_use] extern crate cgmath; /* For Matrix calculation  */
#[macro_use] extern crate json;   /* For JSON parsing        */
#[macro_use] extern crate image;  /* For image decoding      */
#[macro_use] extern crate log;    /* For programming logging */

/* Get the crate's version */
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

/* Use CgMath's structures */
pub use cgmath::*;

/* Logger */
mod logger;
pub use logger::setup_default as setup_logger;
pub use logger::setup as setup_logger_with_level;

/* Import all submodules */
pub mod grid;
pub mod resource;
pub mod graphics;

pub mod physics;
pub use self::physics::Dynamic;

pub mod transform;
pub use self::transform::Camera;

pub mod world;
pub mod game;

pub fn init(){
	if let Err(what) = self::setup_logger() {
		println!("
			ATTENTION! Logger could not be initialized due to the following error, the engine is now
			completely silent: {:?}
		", what);
	}
}
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
	// Setup logger
	let _ = ::setup_logger();

	let mut timer = DeltaTimer::new();
	let _ = timer.duration();

	use std::thread;
	thread::sleep_ms(2000);

	assert_eq!(timer.delta_seconds(), 2);
}

pub trait Update{
	fn update(&mut self, delta: &f64);
}

// ======================= //
// === Automated Tests === //
// ======================= //
#[test]
fn headed_opengl(){
	// Setup logger
	let _ = ::setup_logger();

    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: headed_opengl()")
        .build_glium().unwrap();

    use glium::backend::Facade;
    let context = display.get_context();
    debug!("Built context has: OpenGL {:?}, GLSL {:?}", context.get_opengl_version(), context.get_supported_glsl_version());
}

#[test]
fn render_register_quad(){
	// Setup logger
	let _ = ::setup_logger();

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
	// Setup logger
	let _ = ::setup_logger();

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
