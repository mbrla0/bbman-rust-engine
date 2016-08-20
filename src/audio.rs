use std::io::Read;

/* Import codec libraries */
extern crate hound;
extern crate vorbis;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Codec{ Vorbis, Wave }
impl Codec{
	fn load_sound<R: Read>(&self, source: R) -> usize{
		match self{
			&Codec::Vorbis => {
				0
			},
			&Codec::Wave => {
				0
			}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Format{
	/// Number of channels
	channels: usize,
	/// Number of samples per second
	rate: usize
}
impl Format{
	pub fn bytes_per_second(&self) -> usize{
		use std::mem::size_of;
		self.channels * self.rate * size_of::<i16>()
	}

	pub fn samplerate(&self) -> usize{ self.rate     }
	pub fn channels(&self)   -> usize{ self.channels }
}



#[derive(Debug)]
pub struct Music{

}

#[derive(Debug)]
pub struct Sound{
	/// Audio format
	format: Format,

	/// Raw audio data, in `i16` samples
	data: Vec<i16>
}
impl Sound{
	pub fn new(codec: Codec, source: &str){

	}
}
