use cgmath::{Vector3, Point3};
use super::grid::Grid;

pub trait Dynamic{
	fn set_position(&mut self, x: f64, y: f64, z: f64);
	fn set_rotation(&mut self, x: f64, y: f64, z: f64);
	fn set_dimension(&mut self, x: f64, y: f64, z: f64);

	fn position(&self)   -> &Point3<f64>;
	fn rotation(&self)   -> &Vector3<f64>;
	fn dimensions(&self) -> &Vector3<f64>;

	fn translate(&mut self, x: f64, y: f64, z: f64){
		let translation = self.position().clone();
		self.set_position(translation.x + x, translation.y + y, translation.z + z);
	}

	fn rotate(&mut self, x: f64, y: f64, z: f64){
		let rotation = self.rotation().clone();
		self.set_position(rotation.x + x, rotation.y + y, rotation.z + z);
	}

	fn scale(&mut self, x: f64, y: f64, z: f64){
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
	Ledge(Vector3<f64>) /* Passable only with a given vector */
}

/// A trait describing a physical body
use collision::Aabb3;
pub trait Body: Dynamic{
	/// The shape's bounding box
	fn aabb(&self) -> Aabb3<f64>;

	/// The shape's collision type
	fn collision(&self) -> Collision;

	/// Whether or not this object should ignore the influence from the physiscs of other objects when it comes to its transformation, I.E.: It's "nailed" in place
	fn nailed(&self) -> bool;
}

#[derive(Clone, PartialEq, Debug)]
pub struct DynamicBody{
	position:  Point3<f64>,
	dimension: Vector3<f64>,
	rotation:  Vector3<f64>,

	nailed: bool,
	collision: Option<Aabb3<f64>>
}
impl DynamicBody{
	pub fn new(position: Point3<f64>, dimension: Vector3<f64>, rotation: Vector3<f64>, nailed: bool, collision: Option<Aabb3<f64>>) -> DynamicBody{
		DynamicBody{
			position:  position,
			dimension: dimension,
			rotation:  rotation,

			nailed:    nailed,
			collision: collision
		}
	}

	pub fn update_collision(&mut self, collision: Option<Aabb3<f64>>){
		self.collision = collision;
	}
}
impl Dynamic for DynamicBody{
	fn set_position(&mut self, x: f64, y: f64, z: f64){
		self.position.x = x;
		self.position.y = y;
		self.position.z = z;
	}
	fn set_rotation(&mut self, x: f64, y: f64, z: f64){
		self.rotation.x = x;
		self.rotation.y = y;
		self.rotation.z = z;
	}
	fn set_dimension(&mut self, x: f64, y: f64, z: f64){
		self.dimension.x = x;
		self.dimension.y = y;
		self.dimension.z = z;
	}

	fn position(&self)   -> &Point3<f64>  { &self.position  }
	fn rotation(&self)   -> &Vector3<f64> { &self.rotation  }
	fn dimensions(&self) -> &Vector3<f64> { &self.dimension }
}
impl Body for DynamicBody{
	fn aabb(&self) -> Aabb3<f64> {
		match self.collision{
			Some(ref collision) => collision.clone(),
			None => Aabb3::new(
				Point3::new(self.position.x, self.position.y, self.position.z),
				Point3::new(self.position.x + self.dimension.x, self.position.y + self.dimension.y, self.position.z + self.dimension.z)
			)
		}
	}
	fn collision(&self) -> Collision{ Collision::Solid }
	fn nailed(&self) -> bool { self.nailed }
}

pub struct System<'a>(&'a Vec<Box<Body>>);
impl<'a> System<'a>{
	pub fn new(bodies: &'a Vec<Box<Body>>) -> System<'a>{
		System(bodies)
	}

	pub fn move_body(&mut self, target: &mut Body, vector: Vector3<f64>){
		use cgmath::Zero;
		if vector.is_zero() { return }

		if !target.nailed(){
			// Get the body's properties
			let collision = target.collision();
			let scale     = target.dimensions().clone();
			let position  = target.position().clone();

			// Scan though every pixel in the body that will be able to collide
			// TODO: Find a more efficient to do collision checking
			let mut checked: Option<Point3<f64>> = None;
			for zp in if vector.z == 0.0 { 0..scale.z as usize } else if vector.z.signum() > 0.0 { scale.z as usize - 1..scale.z as usize } else { 0..1 }{
				for yp in if vector.y == 0.0 { 0..scale.y as usize } else if vector.y.signum() > 0.0 { scale.y as usize - 1..scale.y as usize } else { 0..1 }{
					for xp in if vector.x == 0.0 { 0..scale.x as usize } else if vector.x.signum() > 0.0 { scale.x as usize - 1..scale.x as usize } else { 0..1 }{

						// Cast a ray from the current pixel position to the target,
						// checking for any possible collision along the way
						use collision::Ray3;
						use cgmath::InnerSpace;
						let ray = Ray3::new(
							Point3::new(
								xp as f64 + position.x,
								yp as f64 + position.y,
								zp as f64 + position.z
							),
							vector.clone().normalize()
						);

						let mut destination = Point3::new(vector.x, vector.y, vector.z);
						for body in self.0{
							use collision::Intersect;
							if let Some(intersection) = (ray.clone(), body.aabb().clone()).intersection(){
								if destination.x + xp as f64 + position.x > intersection.x {
									destination.x = intersection.x - xp as f64 - position.x;
								}
								if destination.y + yp as f64 + position.y > intersection.y {
									destination.y = intersection.y - yp as f64 - position.y;
								}
								if destination.z + zp as f64 + position.z > intersection.z {
									destination.z = intersection.z - zp as f64 - position.z;
								}
							}
						}

						if let Some(mut check) = checked{
							if check.x > destination.x { check.x = destination.x }
							if check.y > destination.y { check.y = destination.y }
							if check.z > destination.z { check.z = destination.z }

							checked = Some(check)
						}else{
							checked = Some(Point3::new(destination.x, destination.y, destination.z))
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
	}
}
