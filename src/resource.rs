use std::collections::hash_map::{HashMap, Entry};
use std::path::{PathBuf};

use super::graphics::{Texture, TextureProvider};
pub enum Resource{
	Text(String),
	Texture(Texture),
	Raw(Vec<u8>)
}

use std::io::Error as IoError;
use super::graphics::TextureError;
#[derive(Debug)]
pub enum ResourceError{
	DifferentTypeAlreadyCached,
	FileNotFound,
	PathNotAvailable,
	UnableToReadFile(IoError),

	UnableToCreateTexture(TextureError),
}

pub struct Resources{
	root:  PathBuf, /* Root of the resource tree */
	cache:  HashMap<String, Resource>, /* Cached values */
}
use glium::Texture2d;
use glium::backend::Facade;
impl Resources{
	pub fn new(root: &str) -> Resources{
		Resources{
			root:  PathBuf::from(root),
			cache: HashMap::new()
		}
	}

	pub fn texture<F: Facade>(&mut self, facade: &F, uri: &str) -> Result<&Texture2d, ResourceError>{
		match self.cache.entry(uri.to_owned()){
			Entry::Occupied(entry) =>
				if let &mut Resource::Texture(ref texture) = entry.into_mut() { Ok(texture.get_texture()) }
				else {
					error!(r#"Cached element at ID "{}" is not a Resource::Texture"#, uri);
					Err(ResourceError::DifferentTypeAlreadyCached)
				},
			Entry::Vacant(entry) => {
				let mut path_buf = self.root.clone();
				path_buf.push(uri);

				let path = match path_buf.to_str(){
					Some(path) => path,
					None => {
						error!("Could not convert path {:?} to string slice!", path_buf);
						return Err(ResourceError::PathNotAvailable)
					}
				};

				let texture = match Texture::open(facade, path){
					Ok(texture) => texture,
					Err(what)   => return Err(ResourceError::UnableToCreateTexture(what))
				};

				if let &mut Resource::Texture(ref tx) = entry.insert(Resource::Texture(texture)){
					Ok(tx.get_texture())
				}else{ panic!("Iconsistency! Wasn't able to destructure reference to cached value") }
			}
		}
	}

	pub fn text(&mut self, uri: &str) -> Result<&str, ResourceError>{
		match self.cache.entry(uri.to_owned()){
			Entry::Occupied(entry) =>
				if let &mut Resource::Text(ref string) = entry.into_mut() { Ok(string.as_str()) }
				else {
					error!(r#"Cached element at ID "{}" is not a Resource::Text"#, uri);
					Err(ResourceError::DifferentTypeAlreadyCached)
				},
			Entry::Vacant(entry) => {
				let mut path = self.root.clone();
				path.push(uri);

				// Open and read the file into a buffer
				if !path.exists(){
					error!(r#"Path {:?} does not exist"#, path);
					return Err(ResourceError::FileNotFound)
				}
				use std::fs::File;
				use std::io::Read;
				let mut buffer = String::new();
				match File::open(path.clone()){
					Ok(mut file) => {
						if let Err(what) = file.read_to_string(&mut buffer){
							error!(r#"Could not read raw data from path {:?}: {:?}"#, path, what);
							return Err(ResourceError::UnableToReadFile(what))
						}
					},
					Err(what) => {
						error!(r#"Could not open file at path {:?}: {:?}"#, path, what);
						return Err(ResourceError::UnableToReadFile(what))
					}
				}

				if let &mut Resource::Text(ref tx) = entry.insert(Resource::Text(buffer)){
					Ok(tx.as_str())
				}else{ panic!("Iconsistency! Wasn't able to destructure reference to cached value") }
			}
		}
	}

	/** Opens a file and caches it for later reuse, returning a slice reference */
	pub fn raw(&mut self, uri: &str) -> Result<&[u8], ResourceError>{
		match self.cache.entry(uri.to_owned()){
			Entry::Occupied(entry) =>
				if let &mut Resource::Raw(ref buffer) = entry.into_mut() { Ok(&buffer[..]) }
				else {
					error!(r#"Cached element at ID "{}" is not a Resource::Raw"#, uri);
					Err(ResourceError::DifferentTypeAlreadyCached)
				},
			Entry::Vacant(entry) => {
				let mut path = self.root.clone();
				path.push(uri);

				// Open and read the file into a buffer
				if !path.exists(){
					error!(r#"Path {:?} does not exist"#, path);
					return Err(ResourceError::FileNotFound)
				}
				use std::fs::File;
				use std::io::Read;
				let mut buffer = Vec::<u8>::new();
				match File::open(path.clone()){
					Ok(mut file) => {
						if let Err(what) = file.read_to_end(&mut buffer){
							error!(r#"Could not read raw data from path {:?}: {:?}"#, path, what);
							return Err(ResourceError::UnableToReadFile(what))
						}
					},
					Err(what) => {
						error!(r#"Could not open file at path {:?}: {:?}"#, path, what);
						return Err(ResourceError::UnableToReadFile(what))
					}
				}

				if let &mut Resource::Raw(ref buff) = entry.insert(Resource::Raw(buffer)){
					Ok(&buff[..])
				}else{ panic!("Iconsistency! Wasn't able to destructure reference to cached value") }
			}
		}
	}
}

#[cfg(test)]
mod tests{
	use super::Resources;

	#[test]
	fn text(){
		// Setup logger
		let _ = ::setup_logger();

		let mut resources = Resources::new("./test/");
		let file = resources.text("resources/text").unwrap();
		assert_eq!(file, "This is a file used in an automated test, please don't edit it, unless you want the test to fail.\n");
	}

	#[test]
	fn raw(){
		// Setup logger
		let _ = ::setup_logger();

		let mut resources = Resources::new("./test/");
		let file = resources.raw("resources/raw").unwrap();
		assert_eq!(file, &[
			0x00, 0x46, 0x41, 0x5F, 0x44, 0x52, 0x41, 0x47, 0x4F, 0x4E,

			0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
			0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
		]);
	}

	#[test]
	fn texture(){
		// Setup logger
		let _ = ::setup_logger();

		// Setup context
		use glium::DisplayBuild;
		use glium::glutin::WindowBuilder;
		let display = WindowBuilder::new()
			.with_dimensions(1280, 720)
			.with_title("Automated test: resource::texture()")
			.build_glium().unwrap();

		let _ = Resources::new("./test/").texture(&display, "resources/texture.png").unwrap();
	}
}
