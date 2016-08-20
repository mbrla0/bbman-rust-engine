use cgmath::{Point3, Vector3};
use super::physics::Dynamic;

#[derive(Clone, Debug)]
pub struct Actor{
	position:   Point3<f64>,
	dimensions: Vector3<f64>,
	rotation:   Vector3<f64>,
}
impl Dynamic for Actor{
	fn position(&self)   -> &Point3<f64>  { &self.position   }
	fn dimensions(&self) -> &Vector3<f64> { &self.rotation   }
	fn rotation(&self)   -> &Vector3<f64> { &self.dimensions }

	fn set_position(&mut self, position: Point3<f64>)     { self.position = posision }
	fn set_dimensions(&mut self, dimensions: Point3<f64>) { self.dimensions = dimensions }
	fn set_rotation(&mut self, rotation: Point3<f64>)     { self.rotation = rotation }
}
