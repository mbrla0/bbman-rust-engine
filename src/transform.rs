use super::{Vector3, Matrix4, Rad}; // Import cgmath structures

#[derive(Clone, PartialEq)]
pub struct Camera{
	translation: Vector3<f32>,
	rotation:    Vector3<f32>,
	scaling:     Vector3<f32>,
	projection:  Matrix4<f32>,

	stack: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Matrix4<f32>)>
}
impl Camera{
	pub fn new(projection: Matrix4<f32>) -> Camera{
		Camera{
			translation: Vector3{x: 0.0, y: 0.0, z: 0.0},
			rotation:    Vector3{x: 0.0, y: 0.0, z: 0.0},
			scaling:     Vector3{x: 0.0, y: 0.0, z: 0.0},
			projection:  projection,
			stack:       Vec::new()
		}
	}

	pub fn push(&mut self){
		self.stack.push((
			self.translation.clone(),
			self.rotation.clone(),
			self.scaling.clone(),
			self.projection.clone()
		));
	}

	pub fn pop(&mut self){
		if let Some((translation, rotation, scaling, projection)) = self.stack.pop(){
			self.translation = translation;
			self.rotation    = rotation;
			self.scaling     = scaling;
			self.projection  = projection;
		}
	}

	pub fn replace_projection(&mut self, new: Matrix4<f32>) -> Matrix4<f32>{
		use std::mem;
		mem::replace(&mut self.projection, new)
	}

	pub fn translate(&mut self, x: f32, y: f32, z: f32) { self.translation = Vector3{x: x, y: y, z: z}; }
	pub fn rotate(&mut self, x: f32, y: f32, z: f32)    { self.rotation    = Vector3{x: x, y: y, z: z}; }
	pub fn scale(&mut self, x: f32, y: f32, z: f32)     { self.scaling     = Vector3{x: x, y: y, z: z}; }

	pub fn position(&self) -> &Vector3<f32> { &self.translation }
	pub fn rotation(&self) -> &Vector3<f32>    { &self.rotation    }
	pub fn dimensions(&self) -> &Vector3<f32>       { &self.scaling     }

	pub fn get_matrix(&self) -> Matrix4<f32>{
		let translate = Matrix4::from_translation(self.translation);
		let scale = Matrix4::from_nonuniform_scale(self.scaling.x, self.scaling.y, self.scaling.z);

		let rotation = Matrix4::from_angle_x(Rad::new(self.rotation.x))
			* Matrix4::from_angle_y(Rad::new(self.rotation.y))
			* Matrix4::from_angle_z(Rad::new(self.rotation.z));

		// Multiply the calculated matrices
		self.projection * (translate * scale * rotation)
	}
}
