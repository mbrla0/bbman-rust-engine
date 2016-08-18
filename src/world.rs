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

use super::graphics::Texture;
pub struct Room{
    pub images: Vec<Texture>, /* Images available to the room's texture mapping */

    pub blocks:  Grid<Block>, /* Colision mapping */
    pub texture: Grid<usize>  /* Texture mapping  */
}
impl Room{
    pub fn new(block_width: usize, block_height: usize, width: usize, height: usize) -> Option<Room>{
        Some({
			let room = Room{
	            images: Vec::new(),

	            blocks:  Grid::new_with_default(block_width, block_height, 1, width, height, 1, &Block::AIR),
	            texture: Grid::new_with_default(block_width, block_height, 1, width, height, 1, &0)
        	};

			room
		})
    }

	pub fn from_json<F: Facade>(facade: &F, json: &str) -> Option<Room>{
		use json;
		if let json::JsonValue::Object(obj) = match json::parse(json) { Ok(parsed) => parsed, Err(what) => { error!("[Error]Could not parse JSON data: {:?}", what); return None } }{
			Room::from_json_object(facade, obj)
		}else{ None }
	}

    pub fn from_json_object<F: Facade>(facade: &F, obj: json::object::Object) -> Option<Room>{
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
        let mut imgs = Vec::new();
		if let Some(spritesheet) = obj.get("spritesheet"){
			if let Some(path) = spritesheet.as_str(){

				// Load the spritesheet
				let texture = match Texture::open(facade, path){
					Ok(loaded) => loaded,
					Err(what) => {
						error!("Could not load texture: {:?}", what);
						return None
					}
				};

				// Create a new sprite sheet and append it flattened
				if let Ok(sheet) = texture.sprite_sheet(facade, tile_dimension.0, tile_dimension.1){
					let mut flat = sheet.flatten();
					imgs.append(&mut flat);
				}
			}
		} else if let Some(sources) = obj.get("images") {
	        if !sources.is_null(){
		        for source in sources.members(){
					if let Some(stringfied) = sources.as_str(){
						// Load and push the images
						imgs.push(match Texture::open(facade, stringfied){
							Ok(loaded) => loaded,
							Err(what) => {
								error!("Could not load texture: {:?}", what);
								return None
							}
						});
					}
		        }
	        }
		} else {
			warn!("No images have been loaded, this will most likely lead to errors, so, tighten your belts, because this is gonna be a rough landing.")
		}

        // Create a block grid and populate it
        let mut blocks = Grid::new_with_default(tile_dimension.0, tile_dimension.1, 1, grid_dimension.0, grid_dimension.1, 1, &Block::AIR);
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
        let mut texture = Grid::<usize>::new_with_default(tile_dimension.0, tile_dimension.1, 1, grid_dimension.0, grid_dimension.1, 1, &0);
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
        // Setup renderer
		use super::graphics::Renderer2d;
		let mut renderer = match Renderer2d::new(facade, self.texture.width as f32, self.texture.height as f32){
			Some(renderer) => renderer,
            None => {
                error!("Could not create renderer");
                return None
            }
		};

        // Generate a new empty texture
        let result = match Texture2d::empty(facade, self.texture.absolute_width() as u32, self.texture.absolute_height() as u32){
            Ok(texture) => texture,
            Err(err) => {
                error!("Could not create texture: {:?}", err);
                return None
            }
        };

        // Start new lexical context for canvas
        {
            // Get and clear the texture frame buffer
            let mut canvas = result.as_surface();

            use glium::Surface;
            canvas.clear_color(0.0, 0.0, 0.0, 0.0);

            // Render the tiles onto the frame
            for x in 0..self.texture.width{
                for y in 0..self.texture.height{
					// Retrieve the texture that corresponds to the ID
                    let image = &self.images[match self.texture.at(x, y, 0){
                        Some(i) => i.clone(),
                        None => {
                            warn!("Texture mapping error at X: {}, Y: {}", x, y);
                            continue
                        }
                    }];

                    // Draw the tile
					match renderer.sprite(&mut canvas, x as f32, y as f32, 1.0, 1.0, image){
						Ok(_) => {},
						Err(what) => {
							warn!("Could not draw tile at ({}, {}): {}", x, y, what);
						}
					}
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
	// Setup logger
	let _ = ::setup_logger();

	// Setup context
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    let display = WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title("Automated test: room_parse_from_json()")
        .build_glium().unwrap();

	// Create a simple room to be loaded
	let parsed = json::parse(r#"
		{
			"tile_width":  16,
			"tile_height": 16,

			"width":  2,
			"height": 3,

			"spritesheet": "test/sheet.png",
			"blocks":[
				[0, 1],
				[2, 0],
				[1, 2]
			],
			"texture":[
				[1, 0],
				[0, 2],
				[3, 0]
			]
		}
	"#).unwrap();

	// Load the room
	if let json::JsonValue::Object(object) = parsed {
		let room = Room::from_json_object(&display, object).unwrap();

		assert_eq!(room.blocks.elements,  vec![vec![vec![Block::from(0), Block::from(1)], vec![Block::from(2), Block::from(0)], vec![Block::from(1), Block::from(2)]]]);
		assert_eq!(room.texture.elements, vec![vec![vec![1, 0], vec![0, 2], vec![3, 0]]]);
		assert_eq!(room.images.len(), 4);

		assert_eq!(room.blocks.at(1, 0, 0).unwrap().clone(), Block::from(1));
		assert_eq!(room.blocks.at(1, 2, 0).unwrap().clone(), Block::from(2));

		assert_eq!(room.texture.at(0, 0, 0).unwrap().clone(), 1);
		assert_eq!(room.texture.at(1, 2, 0).unwrap().clone(), 0);
		assert_eq!(room.texture.at(1, 1, 0).unwrap().clone(), 2);
		assert_eq!(room.texture.at(0, 2, 0).unwrap().clone(), 3);

		// Render
		room.render(&display);
	} else { panic!("JsonValue is not an object!"); }
}
