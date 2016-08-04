use std::collections::hash_map::{HashMap, Entry};
use std::path::PathBuf;

use super::glium::Texture2d;
pub trait TextureProvider{
	fn get_texture(&self) -> &Texture2d;
}

use glium::texture::{RawImage2d, ClientFormat};
use glium::backend::Facade;
pub enum Texture{
	None,
	Loaded(Texture2d)
}
impl Texture{
	pub fn open<F: Facade>(facade: &F, path: &str) -> Option<Texture>{
		use super::image;
		use std::borrow::Cow;
		if let Ok(img) = image::open(path){
			let rgba = img.to_rgba();
			let data = RawImage2d{
				width:  rgba.width(),
				height: rgba.height(),
				format: ClientFormat::U8U8U8U8,
				data:   Cow::Owned(rgba.into_raw())
			};

			match Texture2d::new(facade, data){
				Ok(texture) => Some(Texture::Loaded(texture)),
				Err(_) => None
			}
		} else { None }
	}

	pub fn open_from_memory<F: Facade>(facade: &F, source: &[u8]) -> Option<Texture>{
		use super::image;
		use std::borrow::Cow;
		if let Ok(img) = image::load_from_memory(source){
			let rgba = img.to_rgba();
			let data = RawImage2d{
				width:  rgba.width(),
				height: rgba.height(),
				format: ClientFormat::U8U8U8U8,
				data:   Cow::Owned(rgba.into_raw())
			};

			match Texture2d::new(facade, data){
				Ok(texture) => Some(Texture::Loaded(texture)),
				Err(_) => None
			}
		} else { None }
	}
}
impl Texture{
	pub fn is_loaded(&self) -> bool{
		match self{
			&Texture::Loaded(_) => true,
			_ => false
		}
	}

	pub fn is_none(&self) -> bool{
		match self{
			&Texture::None => true,
			_ => false
		}
	}
}
impl TextureProvider for Texture{
	fn get_texture(&self) -> &Texture2d{
		match *self{
			Texture::Loaded(ref texture) => &texture,
			_ => panic!("Texture is not loaded")
		}
	}
}

pub struct Resources{
	root:  PathBuf, /* Root of the resource tree */
	cache: HashMap<String, Vec<u8>>, /* Cached values */
}
impl Resources{
	pub fn new(root: PathBuf) -> Resources{
		Resources{
			root:  root,
			cache: HashMap::new()
		}
	}

	/** Opens a file and caches it for later reuse, returning a slice reference */
	pub fn open(&mut self, uri: String) -> Option<&[u8]>{
		// Check if the value has been cached
		match self.cache.entry(String::from(uri.clone())){
			Entry::Occupied(entry) => Some(entry.into_mut().as_slice()),
			Entry::Vacant(entry) => {
				let mut path = self.root.clone();
				path.push(uri);

				// Open and read the file into a buffer
				if !path.exists() { return None }
				use std::fs::File;
				use std::io::Read;
				let mut buffer = Vec::<u8>::new();
				match File::open(path.clone()){
					Ok(mut file) => {
						if let Err(_) = file.read_to_end(&mut buffer){ return None }
					},
					Err(_) => return None
				}

				// Cache it in the main buffer
				Some(entry.insert(buffer).as_slice())
			}
		}
	}

	/** Opens a file and hands over its data, without caching */
	pub fn take(&mut self, uri: String) -> Option<Vec<u8>>{
		match self.cache.entry(uri.clone()){
			Entry::Occupied(entry) => Some(entry.remove()),
			Entry::Vacant(_) => {
				let mut path = self.root.clone();
				path.push(uri);

				// Open and read the file into a buffer
				if !path.exists() { return None }
				use std::fs::File;
				use std::io::Read;
				let mut buffer = Vec::<u8>::new();
				match File::open(path.clone()){
					Ok(mut file) => {
						if let Err(_) = file.read_to_end(&mut buffer){ return None }
					},
					Err(_) => return None
				}

				// Cache it in the main buffer
				Some(buffer)
			}
		}
	}
}

#[test]
fn take(){
	let file   = Resources::new(PathBuf::from("./test/")).take("resource.txt".to_owned()).unwrap();
	let string = String::from_utf8(file).unwrap();

	assert_eq!("This is a file used in an automated test, please don't edit it, unless you want the test to fail.\n", string.as_str())
}

#[test]
fn open(){
	let mut res = Resources::new(PathBuf::from("./test/"));
	let file    = res.open("resource.txt".to_owned()).unwrap();
	let string  = String::from_utf8(Vec::from(file)).unwrap();

	assert_eq!("This is a file used in an automated test, please don't edit it, unless you want the test to fail.\n", string.as_str())
}
