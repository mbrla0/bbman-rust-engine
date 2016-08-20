#[macro_use] pub extern crate glium;     /* For everything OpenGL 3.0+      */
#[macro_use] pub extern crate cgmath;    /* For CG calculations             */
#[macro_use] pub extern crate collision; /* For collision and physics       */
#[macro_use] pub extern crate image;     /* For image decoding              */
#[macro_use] pub extern crate log;       /* For programming logging         */
#[macro_use] pub extern crate json;      /* For JSON parsing                */

/* Get the crate's version */
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

/* Logger */
mod logger;
pub use logger::setup_default as setup_logger;
pub use logger::setup as setup_logger_with_level;

/* Import all submodules */
pub mod time;
pub use self::time::DeltaTimer;

pub mod grid;
pub mod physics;

pub mod audio;
pub mod graphics;

pub mod resource;

pub mod transform;
pub use self::transform::Camera;

/*
 * World is a carryover from very early engine times.
 * At the moment, it hasn't much practical usability, and
 * it planned to be completely remade in the future.
 */
#[deprecated]
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

// ======================= //
// === Automated Tests === //
// ======================= //
#[cfg(test)]
mod tests{
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
}
