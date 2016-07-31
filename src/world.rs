static BASIC_VERTEX_SHADER: &'static str = "
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
static BASIC_FRAGMENT_SHADER: &'static str = "
    #version 140

    in vec2 tex;
    out vec4 color;

    uniform sampler2D bbm_TileTexture;
    void main(){
        color = texture(bbm_TileTexture, tex);
    }
";

use super::{Dynamic, Camera};
use super::grid;
use super::grid::Grid;

use json;
use glium::Texture2d;
use glium::backend::Facade;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Block{
    AIR          = 0,
    SOLID        = 1,
    DESTRUCTABLE = 2
}
impl Block{
	pub fn from(val: usize) -> Block{
		match val{
			0 => Block::AIR,
			1 => Block::SOLID,
			2 => Block::DESTRUCTABLE,
			_ => Block::AIR
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Entity{

}

pub struct Room{
    pub images: Vec<Texture2d>, /* Images available to the room's texture mapping */

    pub blocks:  Grid<Block>, /* Colision mapping */
    pub texture: Grid<usize>  /* Texture mapping  */
}
impl Room{
    pub fn new(block_width: usize, block_height: usize, width: usize, height: usize) -> Option<Room>{
        Some({
			let room = Room{
	            images: Vec::new(),

	            blocks:  match Grid::new_with_default(block_width, block_height, 1, width, height, 1, &Block::AIR) { Some(val) => val, None => return None },
	            texture: match Grid::new_with_default(block_width, block_height, 1, width, height, 1, &0)          { Some(val) => val, None => return None }
        	};

			room
		})
    }

    pub fn from_json<F: Facade>(facade: &F, obj: json::object::Object) -> Option<Room>{
		// Get room dimensions
        let grid_dimension = (
            if let Some(value) = obj.get("width")  { if let Some(int) = value.as_usize() { int } else { return None } } else { return None },
            if let Some(value) = obj.get("height") { if let Some(int) = value.as_usize() { int } else { return None } } else { return None }
		);
        let tile_dimension: (usize, usize) = (
			if let Some(value) = obj.get("tile_width")  { if let Some(int) = value.as_usize() { int } else { return None } } else { return None },
            if let Some(value) = obj.get("tile_height") { if let Some(int) = value.as_usize() { int } else { return None } } else { return None }
        );

		// Load the room's images
        let mut imgs = Vec::<Texture2d>::new();

		if let Some(spritesheet) = obj.get("spritesheet"){
			use json::JsonValue;
			if let &JsonValue::String(ref path) = spritesheet{
				use image;

				// Decode the spritesheet to RGBA
				let buffer = match image::open(path){
					Ok(img) => img.to_rgba(),
					Err(err) => {
						println!("[Error][Room::from_json()] Could not load resource file \"{}\": {:?}", path, err);
						return None
					}
				};

				use std::borrow::Cow;
				use glium::texture::{RawImage2d, ClientFormat};
				let raw_image = RawImage2d{
					format: ClientFormat::U8U8U8U8,
					width:  buffer.width(),
					height: buffer.height(),
					data:   Cow::Owned(buffer.into_raw())
				};

				// Create a new sprite sheet and append it flattened
				if let Some(sheet) = grid::sprite_sheet(facade, tile_dimension.0, tile_dimension.1, raw_image){
					imgs.append(&mut sheet.flatten());
				}
			}
		} else {
	        if let Some(sources) = obj.get("images"){
				if !sources.is_null(){
		            for source in sources.members(){
		                use image;

		                // Decode the image to RGBA
		                let buffer = match image::open(if let Some(string) = source.as_str() { string } else { return None }){
		                    Ok(img) => img.to_rgba(),
		                    Err(err) => {
		                        println!("[Error][Room::from_json()] Could not load resource file \"{}\": {:?}", source, err);
		                        return None
		                    }
		                };

		                use std::borrow::Cow;
		                use glium::texture::{RawImage2d, ClientFormat};
		                let raw_image = RawImage2d{
		                    format: ClientFormat::U8U8U8U8,
		                    width:  buffer.width(),
		                    height: buffer.height(),
		                    data:   Cow::Owned(buffer.into_raw())
		                };

		                // Create a new texture
		                imgs.push(match Texture2d::new(facade, raw_image){
		                    Ok(texture) => texture,
		                    Err(err) => {
		                        println!("[Error][Room::from_json()] Could not upload resource file \"{}\": {:?}", source, err);
		                        return None
		                    }
		                });
		            }
				}
	        }
		}

        // Create a block grid and populate it
        let mut blocks = match Grid::new_with_default(tile_dimension.0, tile_dimension.1, 1, grid_dimension.0, grid_dimension.1, 1, &Block::AIR){
			Some(grid) => grid,
			None => {
				println!("[Error][Room::from_json()] Could not generate blocks grid.");
				return None
			}
		};
		if let Some(obj_blocks) = obj.get("blocks"){
	        for y in 0..grid_dimension.1{
	            for x in 0..grid_dimension.0{
	                if blocks.in_range(x, y, 0) && !obj_blocks[y][x].is_null(){
						if let Some(value) = obj_blocks[y][x].as_usize(){
							blocks.insert(x, y, 0, Block::from(value));
						}
					}
	            }
	        }
		}

		// Create a texture mapping grid and populate it
        let mut texture = match Grid::<usize>::new_with_default(tile_dimension.0, tile_dimension.1, 1, grid_dimension.0, grid_dimension.1, 1, &0){
			Some(grid) => grid,
			None => {
				println!("[Error][Room::from_json()] Could not generate texture grid.");
				return None
			}
		};
		if let Some(obj_texture) = obj.get("texture"){
	        for y in 0..grid_dimension.1{
	            for x in 0..grid_dimension.0{
	                if texture.in_range(x, y, 0) && !obj_texture[y][x].is_null(){
						if let Some(value) = obj_texture[y][x].as_usize(){
							texture.insert(x, y, 0, value);
						}
					}
	            }
	        }
		}

        Some(Room{
            images:  imgs,

			blocks:  blocks,
			texture: texture
        })
    }

    /* Renders the room's texture grid into a single texture */
    pub fn render<F: Facade>(&self, facade: &F) -> Option<Texture2d>{
        // Setup rendering assets
        #[derive(Copy, Clone)]
        struct Vertex{
            position:   [f32; 3],
            tex_coords: [f32; 2]
        }
        implement_vertex!(Vertex, position, tex_coords);

        use glium::VertexBuffer;
        let quad = match VertexBuffer::new(facade, &[
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
        ]){
            Ok(vb)   => vb,
            Err(err) => {
                println!("[Error][Room::render()] Could not register quad vertex buffer: {:?}", err);
                return None
            }
        };

        use glium::Program;
        let shader = match Program::from_source(facade, BASIC_VERTEX_SHADER, BASIC_FRAGMENT_SHADER, None){
            Ok(program) => program,
            Err(err) => {
                println!("[Error][Room::render()] Could not compile grid rendering shader: {:?}", err);
                return None
            }
        };

        // Generate a new empty texture
        let result = match Texture2d::empty(facade, self.texture.absolute_width() as u32, self.texture.absolute_height() as u32){
            Ok(texture) => texture,
            Err(err) => {
                println!("[Error][Room::render()] Could not create texture: {:?}", err);
                return None
            }
        };

        // Start new lexical context for canvas
        {
            // Get and clear the texture frame buffer
            let mut canvas = result.as_surface();

            use glium::Surface;
            canvas.clear_color(0.0, 0.0, 0.0, 0.0);

            // Create a camera for rendering the romm
            use cgmath;
            let mut camera = Camera::new(cgmath::ortho(0.0, self.texture.absolute_width() as f32, self.texture.absolute_height() as f32, 0.0, -1.0, 20.0));
			camera.scale(self.texture.tile_width as f32, self.texture.tile_height as f32, 1.0);

            // Render the tiles onto the frame
            for x in 0..self.texture.width{
                for y in 0..self.texture.height{
					println!("Processing for {:?} at ({}, {})", self.texture.at(x, y, 0), x, y);

					// Retrieve the texture that corresponds to the ID
                    let image = &self.images[match self.texture.at(x, y, 0){
                        Some(i) => i.clone(),
                        None => {
                            println!("[Warning][Room::render()] Texture mapping error at X: {}, Y: {}", x, y);
                            continue
                        }
                    }];

                    // Move camera to match block
                    camera.translate(x as f32, y as f32, 0.0);
					let matrix: [[f32; 4]; 4] = camera.get_matrix().into();

					println!("Rendering with matrix: {:?}", matrix);

                    // Draw the tile
                    use glium::index::{NoIndices, PrimitiveType};
					use glium::uniforms::MagnifySamplerFilter;
                    let _ = canvas.draw(
                        &quad,
                        &NoIndices(PrimitiveType::TriangleStrip),
                        &shader,
                        &uniform!{
                            bbm_GridMatrix:  matrix,
                            bbm_TileTexture: image.sampled().magnify_filter(MagnifySamplerFilter::Nearest)
                        },
                        &Default::default()
                    );
                }
            }

            // Canvas gets discarded here
        }

        // Output the result
        Some(result)
    }
}

// ======================= //
// === Automated Tests === //
// ======================= //

#[test]
fn room_parse_from_json(){
	// Setup context
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: room_parse_from_json()")
        .build_glium().unwrap();

	// Create a simple room to be loaded
	let parsed = json::parse("
		{
			\"tile_width\":  2,
			\"tile_height\": 2,

			\"width\":  2,
			\"height\": 3,

			\"images\":[\"res/test/0.png\", \"res/test/1.png\"],
			\"blocks\":[
				[0, 1],
				[2, 0],
				[1, 2]
			],
			\"texture\":[
				[1, 0],
				[0, 1],
				[1, 0]
			]
		}
	").unwrap();

	// Load the room
	if let json::JsonValue::Object(object) = parsed {
		let room = Room::from_json(&display, object).unwrap();

		assert_eq!(room.blocks.elements,  vec![vec![vec![Block::from(0), Block::from(1)], vec![Block::from(2), Block::from(0)], vec![Block::from(1), Block::from(2)]]]);
		assert_eq!(room.texture.elements, vec![vec![vec![1, 0], vec![0, 1], vec![1, 0]]]);
		assert_eq!(room.images.len(), 2);

		assert_eq!(room.blocks.at(1, 0, 0).unwrap().clone(), Block::from(1));
		assert_eq!(room.blocks.at(1, 2, 0).unwrap().clone(), Block::from(2));

		assert_eq!(room.texture.at(0, 0, 0).unwrap().clone(), 1);
		assert_eq!(room.texture.at(1, 2, 0).unwrap().clone(), 0);
	}else { panic!("JsonValue is not an object!"); }
}
