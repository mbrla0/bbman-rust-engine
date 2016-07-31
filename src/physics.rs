use super::{Vector3, Matrix4};
use super::grid::Grid;

pub trait Dynamic{
	fn translate(&mut self, x: f32, y: f32, z: f32);
	fn rotate(&mut self, x: f32, y: f32, z: f32);
	fn scale(&mut self, x: f32, y: f32, z: f32);

	fn get_translation(&self) -> &Vector3<f32>;
	fn get_rotation(&self) -> &Vector3<f32>;
	fn get_scale(&self) -> &Vector3<f32>;

	fn get_matrix(&self) -> Matrix4<f32>;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Colision{
	AIR,                /* Completely passable   */
	SOLID,              /* Completely impassable */
	TRAP,               /* Passable, trapping a body once it's completely inside */
	FLUID(usize),       /* Passable, slowing movement by a given factor */
	LEDGE(Vector3<f32>) /* Passable only with a given vector */
}

pub trait Body: Dynamic{
	fn colision(&self) -> Grid<Colision>; /* The grid of colision elements for this body */
	fn nailed(&self)   -> bool; /* Whether or not this object should ignore the influence from the physiscs of other objects when it comes to its transformation, I.E.: It's "nailed" in place */
}
