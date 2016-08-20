use cgmath::{Vector3, Point3};
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
	Fluid(f64),         /* Passable, slowing movement by a given factor */
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
	boundaries: Option<Aabb3<f64>>,
	collision:  Option<Collision>
}
impl DynamicBody{
	pub fn new(position: Point3<f64>, dimension: Vector3<f64>, rotation: Vector3<f64>, nailed: bool, boundaries: Option<Aabb3<f64>>, collision: Option<Collision>) -> DynamicBody{
		DynamicBody{
			position:  position,
			dimension: dimension,
			rotation:  rotation,

			nailed:     nailed,
			boundaries: boundaries,
			collision:  collision
		}
	}

	pub fn update_boundaries(&mut self, boundaries: Option<Aabb3<f64>>){
		self.boundaries = boundaries;
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
		match self.boundaries{
			Some(ref collision) => collision.clone(),
			None => Aabb3::new(
				Point3::new(self.position.x, self.position.y, self.position.z),
				Point3::new(self.position.x + self.dimension.x, self.position.y + self.dimension.y, self.position.z + self.dimension.z)
			)
		}
	}

	fn collision(&self) -> Collision{
		match self.collision{
			Some(ref c) => c.clone(),
			None => Collision::Solid
		}
	}

	fn nailed(&self) -> bool { self.nailed }
}

fn _modulus(n: f64) -> f64{
	if n < 0.0 { -n } else { n }
}

/* Implement intersection for two Aabb3's */
fn aabb_aabb_intersection(a: &Aabb3<f64>, b: &Aabb3<f64>) -> bool{
	   (a.max.x >= b.min.x && b.max.x >= a.min.x)
	&& (a.max.y >= b.min.y && b.max.y >= a.min.y)
	&& (a.max.z >= b.min.z && b.max.z >= a.min.z)
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
			// Scan though every possible pixel the body will be able to collide with
			let mut distance = vector.clone();
			while distance != Vector3::new(0.0, 0.0, 0.0){
				let movement = Vector3::new(
					if distance.x == 0.0 { 0.0 } else if _modulus(distance.x) < 1.0 { distance.x } else { 1.0 },
					if distance.y == 0.0 { 0.0 } else if _modulus(distance.y) < 1.0 { distance.y } else { 1.0 },
					if distance.z == 0.0 { 0.0 } else if _modulus(distance.z) < 1.0 { distance.z } else { 1.0 }
				);

				// Check for any collisions
				let mut checked = vector.clone();
				for body in self.0{
					macro_rules! check{
						($c:ident) => ({
							// Check for $c collision
							let mut aabb = target.aabb().clone();
							aabb.min.$c += movement.$c - 1.0;
							aabb.max.$c += movement.$c - 1.0;
							if !aabb_aabb_intersection(&aabb, &body.aabb()){
								if movement.$c < checked.$c { checked.$c = movement.$c }
							} else {
								let moved = match body.collision(){
									Collision::Air   => movement.$c,
									Collision::Solid => 0.0,
									Collision::Fluid(drag) => { distance.$c -= movement.$c; movement.$c / drag },
									Collision::Ledge(vec) => {
										if vec.$c.signum() == movement.$c.signum(){
											movement.$c
										} else { 0.0 }
									}
								};
								if moved < checked.$c { checked.$c = moved }
							}
						};)
					};

					// Check for all axis
					check!(x); check!(y); check!(z);
				}

				match checked{
					Vector3{x: 0.0, y: 0.0, z: 0.0} => break,
					_ => {
						distance -= checked;
						target.translate(checked.x, checked.y, checked.z);
					}
				}
			}
		}
	}
}
