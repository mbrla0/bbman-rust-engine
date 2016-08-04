#[derive(Copy, Clone)]
struct Vert{
	bbm_Position: [f32; 3],
	bbm_TexCoord: [f32; 2],
	bbm_Normal:   [f32; 3]
}
implement_vertex!(Vertex, bbm_Position, bbm_TexCoord, bbm_Normal);

use super::glium::backend::Facade;
use super::glium::{VertexBuffer, IndexBuffer, Vertex};
use super::glium::index::{Index, PrimitiveType};
struct Model<V: Vertex, I: Index>{
	vertices: VertexBuffer<V>,
	indices:  Option<IndexBuffer<I>>,
}
impl<V, I> Model<V, I>{
	pub fn new<F: Facade, V: Vertex, I: Index>(facade: &F, vertices: &[V], indices: Option<&[I]>, primitive: PrimitiveType) -> Option<Model<V, I>>{
		let vb = VertexBuffer::new(facade, vertices);
		let ib = match indices{
			Some(indices) => Some(IndexBuffer::new(facade, indices, primitive)),
			None => {
				use super::glium::index::NoIndices;
				NoIndices(primitive)
			}
		};

	}
}


static Renderer2d_VertexShader: &'static str = "
	#version 140

	in vec3 bbm_Position;
	in vec2 bbm_TexCoord;
	in vec3 bbm_Normal;

	out vec2 texcoord;

	void main(){ gl_Position = vec4(bbm_Position, 1); }
";
static Renderer2d_FragmentShader: &'static str = "
	#version 140

	in vec2 texcoord;
	out vec4 color;

	uniform vec4 bbm_Color;
	uniform sampler2D bbm_Texture;

	void main(){
		color = texture(bbm_Texture, texcoord);
	}
";

use super::glium::Surface;
pub struct Renderer2d<S: Surface>{
	target: S,
	rectangle: VertexBuffer
}
impl<S> Renderer2d<S> where S: Surface{
	pub fn wrap(value: S) -> Renderer2d<S>{
		Renderer2d{
			target: value,
			rectangle: VertexBuffer::new(&value, &[
				Vertex{ bbm_Position: [0.0, 0.0, 0.0], bbm_TexCoord: [0.0, 1.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [1.0, 0.0, 0.0], bbm_TexCoord: [1.0, 1.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [1.0, 1.0, 0.0], bbm_TexCoord: [1.0, 0.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [0.0, 1.0, 0.0], bbm_TexCoord: [0.0, 0.0], bbm_Normal: [0.0, 0.0, 0.0] },
				Vertex{ bbm_Position: [0.0, 0.0, 0.0], bbm_TexCoord: [0.0, 1.0], bbm_Normal: [0.0, 0.0, 0.0] }
			])
		}
	}
}
