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
    pub fn new(tile_width: usize, tile_height: usize, tile_depth: usize, width: usize, height: usize, depth: usize) -> Grid<T>{
        let mut tmp = Grid::<T>{
            tile_width:  tile_width,
            tile_height: tile_height,
			tile_depth:  tile_depth,

            width:  width,
            height: height,
			depth:  depth,

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
        tmp
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
	pub fn new_with_default(tile_width: usize, tile_height: usize, tile_depth: usize, width: usize, height: usize, depth: usize, default: &T) -> Grid<T>{
		let mut tmp = Grid::<T>::new(tile_width, tile_height, tile_depth, width, height, depth);

		// Clear the grid
		tmp.clear(default);

		// Return the new grid
		tmp
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
impl<T: Copy> Grid<T>{
	pub fn resize(&mut self, width: usize, height: usize, depth: usize, value: T){
		self.elements.resize(depth, Vec::<Vec<T>>::new());
		for mut sheet in &mut self.elements{
			sheet.resize(height, Vec::<T>::new());
			for row in sheet{
				row.resize(width, value);
			}
		}
	}
}

#[cfg(test)]
mod tests{
	use super::Grid;

	#[test]
	fn size(){
		// Setup logger
		let _ = ::setup_logger();

		// Create a new grid
		let grid = Grid::<u8>::new_with_default(2, 2, 2, 3, 2, 1, &0);

		assert_eq!(grid.elements.len(),       grid.depth);
		assert_eq!(grid.elements[0].len(),    grid.height);
		assert_eq!(grid.elements[0][0].len(), grid.width);
	}

	#[test]
	fn flatten(){
		// Setup logger
		let _ = ::setup_logger();

		let mut grid = Grid::<u8>::new_with_default(2, 2, 2, 2, 2, 2, &0);

		grid.insert(0, 0, 1, 32);  grid.insert(1, 0, 1, 64);
		grid.insert(0, 1, 1, 128); grid.insert(1, 1, 1, 256);
		grid.insert(0, 0, 0, 2);   grid.insert(1, 0, 0, 4);
		grid.insert(0, 1, 0, 8);   grid.insert(1, 1, 0, 16);

		assert_eq!(grid.flatten(), vec![2, 4, 8, 16, 32, 64, 128, 256]);
	}
}
