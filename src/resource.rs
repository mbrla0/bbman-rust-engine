use std::collections::hash_map::{HashMap, Entry};
use std::path::{PathBuf, Path};

pub struct Resources{
	root:  PathBuf, /* Root of the resource tree */
	cache: HashMap<String, Vec<u8>>, /* Cached values */
}
impl Resources{
	pub fn new(root: &Path) -> Resources{
		Resources{
			root:  PathBuf::from(root),
			cache: HashMap::new()
		}
	}

	/** Opens a file and caches it for later reuse, returning a slice reference */
	pub fn open(&mut self, uri: &str) -> Option<&[u8]>{
		// Check if the value has been cached
		match self.cache.entry(String::from(uri)){
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
	pub fn take(&mut self, uri: &str) -> Option<Vec<u8>>{
		match self.cache.entry(String::from(uri)){
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
	let file   = Resources::new(Path::new("./test/")).take("resource.txt").unwrap();
	let string = String::from_utf8(file).unwrap();

	assert_eq!("This is a file used in an automated test, please don't edit it, unless you want the test to fail.\n", string.as_str())
}

#[test]
fn open(){
	let mut res = Resources::new(Path::new("./test/"));
	let file    = res.open("resource.txt").unwrap();
	let string  = String::from_utf8(Vec::from(file)).unwrap();

	assert_eq!("This is a file used in an automated test, please don't edit it, unless you want the test to fail.\n", string.as_str())
}
