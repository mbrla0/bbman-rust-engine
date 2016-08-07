#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Grid<T>{
    pub tile_width:  usize,
    pub tile_height: usize,
	pub tile_depth:  usize,

    pub width:  usize,
    pub height: usize,
	pub depth:  usize,

    pub elements: Vec<Vec<Vec<T>>>
}
impl<T> Grid<T>{
    pub fn new(tile_width: usize, tile_height: usize, tile_depth: usize, width: usize, height: usize, depth: usize) -> Option<Grid<T>>{
        let mut tmp = Grid::<T>{
            tile_width:  match tile_width  { 0 => return None, n @ _ => n },
            tile_height: match tile_height { 0 => return None, n @ _ => n },
			tile_depth:  match tile_depth  { 0 => return None, n @ _ => n },

            width:  match width  { 0 => return None, n @ _ => n },
            height: match height { 0 => return None, n @ _ => n },
			depth:  match depth  { 0 => return None, n @ _ => n },

            elements: Vec::with_capacity(depth)
        };

		// Fill the grid with sheets
		for _ in 0..depth{
			let mut sheet = Vec::<Vec<T>>::with_capacity(height);

			// Fill the sheet with rows
			for _ in 0..height{ sheet.push(Vec::with_capacity(width)) }

			tmp.elements.push(sheet);
		}

        // Return the new grid
        Some(tmp)
    }

	/* Converts the grid into a linar vector, with rows being inseted from top to bottom, front to back, consuming the grid */
	pub fn flatten(mut self) -> Vec<T>{
		let mut result = Vec::<T>::with_capacity(self.width * self.height * self.depth);
		for z in 0..self.depth{
			for y in 0..self.height{ result.append(&mut self.elements[z][y]); }
		}

		result
	}

	pub fn in_range(&self, x: usize, y: usize, z: usize) -> bool {
		if self.height > y && self.width > x && self.depth > z{
            true
        } else { false }
	}

    pub fn at(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        if self.in_range(x, y, z){
            Some(&self.elements[z][y][x])
        } else { None }
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
		if self.in_range(x, y, z){
            Some(&mut self.elements[z][y][x])
        } else { None }
    }

	pub fn push(&mut self, y: usize, z: usize, value: T) {
		if self.in_range(0, y, z){
			self.elements[z][y].push(value);
		}
	}

    /* Return the absolute width of the grid in pixels */
    pub fn absolute_width(&self)  -> usize { self.tile_width  * self.width  }

    /* Return the absolute width of the grid in pixels */
    pub fn absolute_height(&self) -> usize { self.tile_height * self.height }

	/* Return the absolute depth of the grid in pixels */
	pub fn absolute_depth(&self)  -> usize { self.tile_depth  * self.depth  }
}
impl<T: Clone> Grid<T>{
	pub fn new_with_default(tile_width: usize, tile_height: usize, tile_depth: usize, width: usize, height: usize, depth: usize, default: &T) -> Option<Grid<T>>{
		let mut tmp = Grid::<T>{
            tile_width:  match tile_width  { 0 => return None, n @ _ => n },
            tile_height: match tile_height { 0 => return None, n @ _ => n },
			tile_depth:  match tile_depth  { 0 => return None, n @ _ => n },

            width:  match width  { 0 => return None, n @ _ => n },
            height: match height { 0 => return None, n @ _ => n },
			depth:  match depth  { 0 => return None, n @ _ => n },

            elements: Vec::with_capacity(depth)
        };

		// Clear the grid
		tmp.clear(default);

		// Return the new grid
		Some(tmp)
	}

	pub fn clear(&mut self, value: &T){
		// Clear the current grid
		self.elements.clear();

		// Fill the grid with new elements
		for _ in 0..self.depth{
			let mut sheet = Vec::<Vec<T>>::with_capacity(self.height);

			for _ in 0..self.height{
		        let mut row = Vec::<T>::with_capacity(self.width);
		        for _ in 0..self.width{ row.push(value.clone()) }

				sheet.push(row);
		    }

			self.elements.push(sheet);
		}
	}

	pub fn insert(&mut self, x: usize, y: usize, z: usize, value: T){
		if self.in_range(x, y, z){
			self.elements[z][y][x] = value;
		}
	}
}

/* Create a type alias and initializer for sprite sheets */
pub type SpriteGrid = Grid<Texture2d>;

use glium::Texture2d;
use glium::backend::Facade;
use glium::texture::{PixelValue, RawImage2d};
pub fn sprite_sheet<F: Facade, P: PixelValue>(facade: &F, sprite_width: usize, sprite_height: usize, sheet: RawImage2d<P>) -> Option<SpriteGrid>{
	// Get the grid's dimensions in sprites
	let dimensions = (
		(sheet.width as f64  / sprite_width  as f64).floor() as usize,
		(sheet.height as f64 / sprite_height as f64).floor() as usize
	);

	// Read the sheet's data into sectors
	use std::mem;
	match Grid::<Texture2d>::new(sprite_width, sprite_height, 1, dimensions.0, dimensions.1, 1) {
		Some(mut grid) => {
			for y in 0..dimensions.1{
				for x in 0..dimensions.0{
					// Get the pixel data for the current sprite
					let mut sprite = Vec::<P>::with_capacity(sprite_width * sprite_height);
					for b in 0..sprite_height{
						for a in 0..sprite_width{
							let bpp = sheet.format.get_size() / mem::size_of::<P>();
							for e in 0..bpp{
								let offset_y = y * sprite_height + sprite_height - 1 - b;
								let offset_x = x * sprite_width  + a;

								// Calculate the offset
								let offset = offset_y * sheet.width as usize + offset_x;

								sprite.push(sheet.data[offset * bpp + e]);
							}
						}
					}

					use std::borrow::Cow;
					grid.push(y, 0, match Texture2d::new(facade, RawImage2d{
						format: sheet.format,
						width:  sprite_width  as u32,
						height: sprite_height as u32,
						data:   Cow::Owned(sprite)
					}){
						Ok(texture) => texture,
						Err(err) => {
							error!("Could not create texture for sprite at ({}, {}): {:?}", x, y, err);
							return None
						}
					});
				}
			}

			Some(grid)
		},
		None => return None
	}
}

// ======================= //
// === Automated Tests === //
// ======================= //
#[test]
fn size(){
	// Setup logger
	let _ = ::setup_logger();

    // Create a new grid
    let grid = Grid::<u8>::new_with_default(2, 2, 2, 3, 2, 1, &0).unwrap();

    assert_eq!(grid.elements.len(),       grid.depth);
    assert_eq!(grid.elements[0].len(),    grid.height);
	assert_eq!(grid.elements[0][0].len(), grid.width);
}

#[test]
#[should_panic]
fn dimention_is_zero(){
	// Setup logger
	let _ = ::setup_logger();

    // Create a new grid with passing one dimention to be 0
    let grid = Grid::<u8>::new_with_default(0, 2, 0, 2, 2, 2, &0).unwrap();
}

#[test]
fn flatten(){
	// Setup logger
	let _ = ::setup_logger();

	let mut grid = Grid::<u8>::new_with_default(2, 2, 2, 2, 2, 2, &0).unwrap();

	grid.insert(0, 0, 1, 32);  grid.insert(1, 0, 1, 64);
	grid.insert(0, 1, 1, 128); grid.insert(1, 1, 1, 256);
	grid.insert(0, 0, 0, 2);   grid.insert(1, 0, 0, 4);
	grid.insert(0, 1, 0, 8);   grid.insert(1, 1, 0, 16);

	assert_eq!(grid.flatten(), vec![2, 4, 8, 16, 32, 64, 128, 256]);
}

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
	use image;

	// Decode the image to RGBA
	let buffer = image::open("test/sheet.png").unwrap().to_rgba();

	use std::borrow::Cow;
	use glium::texture::{RawImage2d, ClientFormat};
	let raw_image = RawImage2d{
		format: ClientFormat::U8U8U8U8,
		width:  buffer.width(),
		height: buffer.height(),
		data:   Cow::Owned(buffer.into_raw())
	};

	let spritesheet = sprite_sheet(&display, 16, 16, raw_image).unwrap();

	assert_eq!(spritesheet.elements.len(),       1);
	assert_eq!(spritesheet.elements[0].len(),    2);
	assert_eq!(spritesheet.elements[0][0].len(), 2);
	assert!(spritesheet.in_range(1, 1, 0));
	assert_eq!(spritesheet.flatten().len(), 4);
}
