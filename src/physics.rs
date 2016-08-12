use super::{Vector3, Matrix4};
use super::grid::Grid;

pub trait Dynamic{
	fn set_position(&mut self, x: f32, y: f32, z: f32);
	fn set_rotation(&mut self, x: f32, y: f32, z: f32);
	fn set_dimension(&mut self, x: f32, y: f32, z: f32);

	fn position(&self)   -> &Vector3<f32>;
	fn rotation(&self)   -> &Vector3<f32>;
	fn dimensions(&self) -> &Vector3<f32>;

	fn translate(&mut self, x: f32, y: f32, z: f32){
		let translation = self.position().clone();
		self.set_position(translation.x + x, translation.y + y, translation.z + z);
	}

	fn rotate(&mut self, x: f32, y: f32, z: f32){
		let rotation = self.rotation().clone();
		self.set_position(rotation.x + x, rotation.y + y, rotation.z + z);
	}

	fn scale(&mut self, x: f32, y: f32, z: f32){
		let scale = self.dimensions().clone();
		self.set_position(scale.x + x, scale.y + y, scale.z + z);
	}
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Collision{
	Air,                /* Completely passable   */
	Solid,              /* Completely impassable */
	Trap,               /* Passable, trapping a body once it's completely inside */
	Fluid(usize),       /* Passable, slowing movement by a given factor */
	Ledge(Vector3<f32>) /* Passable only with a given vector */
}

pub trait Body: Dynamic{
	fn collision(&self) -> Grid<Collision>; /* The grid of collision elements for this body */
	fn nailed(&self)    -> bool; /* Whether or not this object should ignore the influence from the physiscs of other objects when it comes to its transformation, I.E.: It's "nailed" in place */
}

use cgmath::Point3;
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Line{
	p1: Point3<f32>,
	p2: Point3<f32>
}
impl Line{
	pub fn new(p1: Point3<f32>, p2: Point3<f32>) -> Line{
		Line{ p1: p1, p2: p2 }
	}

	fn _modulus(value: f32) -> f32{
		if value < 0.0 { -value } else { value }
	}

	/** Casts a ray from p1 to p2, running a closure for every stop */
	pub fn raycast<F>(self, test: F) -> Line where F: Fn(&Point3<f32>, &Vector3<f32>) -> Vector3<f32>{
		// Calculate the direction vector
		let mut distance = self.distance();

		// Trace the line
		let mut point = self.p1.clone();
		while point != self.p2{
			let vector = Vector3::new(
				if distance.x == 0.0 { 0.0 } else if Line::_modulus(distance.x) < 1.0 { distance.x } else { distance.x.signum() },
				if distance.y == 0.0 { 0.0 } else if Line::_modulus(distance.y) < 1.0 { distance.y } else { distance.y.signum() },
				if distance.z == 0.0 { 0.0 } else if Line::_modulus(distance.z) < 1.0 { distance.z } else { distance.z.signum() }
			);

			let result = match test(&point, &vector){
				Vector3{x: 0.0, y: 0.0, z: 0.0} => break, /* In case there is no movement, stop loop */
				vec @ _ => vec
			};

			point    += result;
			distance -= result;
		}

		Line{
			p1: self.p1,
			p2: point
		}
	}

	pub fn distance(&self) -> Vector3<f32>{
		self.p2.clone() - self.p1.clone()
	}
}
#[test]
fn line_retrace(){
	let mut line = Line::new(Point3::new(10.09, 20.15, 20.30), Point3::new(26.10, 20.15, 14.00));
	line = line.raycast(|position, vector|{
		if position.x.floor() == 20.0{ Vector3::new(0.0, vector.y, vector.z) } else { vector.clone() }
	});

	assert_eq!(line.p2, Point3::new(20.09, 20.15, 14.00));
}

#[derive(Clone, PartialEq, Debug)]
pub struct DynamicBody{
	position:  Vector3<f32>,
	dimension: Vector3<f32>,
	rotation:  Vector3<f32>,

	nailed: bool,
	collision: Option<Grid<Collision>>
}
impl DynamicBody{
	pub fn new(position: Vector3<f32>, dimension: Vector3<f32>, rotation: Vector3<f32>, nailed: bool, collision: Option<Grid<Collision>>) -> DynamicBody{
		DynamicBody{
			position:  position,
			dimension: dimension,
			rotation:  rotation,

			nailed:    nailed,
			collision: collision
		}
	}

	pub fn update_collision(&mut self, collision: Option<Grid<Collision>>){
		self.collision = collision;
	}
}
impl Dynamic for DynamicBody{
	fn set_position(&mut self, x: f32, y: f32, z: f32){
		self.position.x = x;
		self.position.y = y;
		self.position.z = z;
	}
	fn set_rotation(&mut self, x: f32, y: f32, z: f32){
		self.rotation.x = x;
		self.rotation.y = y;
		self.rotation.z = z;
	}
	fn set_dimension(&mut self, x: f32, y: f32, z: f32){
		self.dimension.x = x;
		self.dimension.y = y;
		self.dimension.z = z;
	}

	fn position(&self)   -> &Vector3<f32> { &self.position  }
	fn rotation(&self)   -> &Vector3<f32> { &self.rotation  }
	fn dimensions(&self) -> &Vector3<f32> { &self.dimension }
}
impl Body for DynamicBody{
	fn collision(&self) -> Grid<Collision> {
		match self.collision{
			Some(ref collision) => collision.clone(),
			None => Grid::new(self.dimensions().x as usize, self.dimensions().y as usize, self.dimensions().z as usize, 1, 1, 1)
		}
	}
	fn nailed(&self) -> bool { self.nailed }
}

use std::collections::HashMap;
pub struct System{
	bodies: HashMap<String, Box<Body>>,
}
impl System{
	pub fn new() -> System{
		System{
			bodies: HashMap::new()
		}
	}

	pub fn insert(&mut self, id: &str, body: Box<Body>) -> Option<Box<Body>>{
		self.bodies.insert(id.to_owned(), body)
	}

	pub fn remove(&mut self, id: &str) -> Option<Box<Body>>{
		self.bodies.remove(id)
	}

	pub fn get(&self, id: &str) -> Option<&Box<Body>>{
		self.bodies.get(id)
	}

	pub fn bodies(&self) -> &HashMap<String, Box<Body>>{
		&self.bodies
	}

	fn collision_at(&self, x: f32, y: f32, z: f32) -> Collision{
		for body in self.bodies.values(){
			let collision = body.collision();

			let translation = body.position();
			let scale = body.dimensions();
			if translation.x <= x && x <= translation.x + scale.x &&
			   translation.y <= y && y <= translation.y + scale.y &&
			   translation.z <= z && z <= translation.z + scale.z
			{
				let ratios = (
					collision.width  as f32 / scale.x,
					collision.height as f32 / scale.y,
					collision.depth  as f32 / scale.z
				);

				return match collision.at(
					((x - translation.x) * ratios.0).floor() as usize,
					((y - translation.y) * ratios.1).floor() as usize,
					((z - translation.z) * ratios.2).floor() as usize){

					Some(entry) => entry.clone(),
					None => { warn!("Could not map coordinates to collision grid"); Collision::Air }
				}
			}
		}
		Collision::Air
	}

	pub fn move_body(&mut self, id: &str, vector: Vector3<f32>){
		use cgmath::Zero;
		if vector.is_zero() { return }

		let mut target = match self.bodies.remove(id){
			Some(body) => body,
			None => {
				warn!("
					Tried to move object with id {}, which is not present in the current system.
					Available items are: {:?}
				", id, self.bodies.keys().collect::<Vec<&String>>());

				return
			}
		};

		if !target.nailed(){
			// Get the body's properties
			let collision = target.collision(); /* TODO: Use the body's collision */
			let scale     = target.dimensions().clone();
			let position  = target.position().clone();

			// Scan though every pixel in the body that will be able to collide
			// TODO: Find a more efficient to do collision checking
			let mut checked: Option<Point3<f32>> = None;
			for zp in if vector.z == 0.0 { 0..scale.z as usize } else if vector.z.signum() > 0.0 { scale.z as usize - 1..scale.z as usize } else { 0..1 }{
				for yp in if vector.y == 0.0 { 0..scale.y as usize } else if vector.y.signum() > 0.0 { scale.y as usize - 1..scale.y as usize } else { 0..1 }{
					for xp in if vector.x == 0.0 { 0..scale.x as usize } else if vector.x.signum() > 0.0 { scale.x as usize - 1..scale.x as usize } else { 0..1 }{

						// Cast a ray from the current pixel position to the target,
						// checking for any possible collision along the way
						let line = Line {
							p1: Point3::new(
								xp as f32 + position.x,
								yp as f32 + position.y,
								zp as f32 + position.z
							),
							p2: Point3::new(
								xp as f32 + position.x + vector.x,
								yp as f32 + position.y + vector.y,
								zp as f32 + position.z + vector.z
							)
						}.raycast(|position, vector|{
							match self.collision_at(position.x + vector.x, position.y + vector.y, position.z + vector.z){
								Collision::Air          => vector.clone(),       /* Don't change the vector           */
								Collision::Fluid(drag)  => vector / drag as f32, /* Divide movement by drag           */
								Collision::Ledge(ledge) => {                     /* Multipĺy vector by ledge diretion */
									Vector3::new(
										vector.x * ledge.x,
										vector.y * ledge.y,
										vector.z * ledge.z
									)
								},
								_ => Vector3::new(0.0, 0.0, 0.0)
							}
						});

						if let Some(mut check) = checked{
							let distance = line.distance();

							if check.x > distance.x { check.x = distance.x }
							if check.y > distance.y { check.y = distance.y }
							if check.z > distance.z { check.z = distance.z }

							checked = Some(check)
						}else{
							let distance = line.distance();
							checked = Some(Point3::new(distance.x, distance.y, distance.z))
						}
					}
				}
			}

			// Apply movement based on how far the ray traveled
			match checked {
				Some(checked) => target.translate(checked.x, checked.y, checked.z),
				None => target.translate(vector.x, vector.y, vector.z)
			}
		}

		// Re-store target
		self.insert(id, target);
	}
}
