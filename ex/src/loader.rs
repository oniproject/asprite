#![allow(dead_code)]

use std::fs::File;
use std::path::PathBuf;

use std::sync::Arc;

error_chain! {
	/*foreign_links {
		FromUtf8(FromUtf8Error) #[doc = "Wraps a UTF-8 error"];
		Utf8(Utf8Error) #[doc = "Wraps a UTF-8 error"];
	}*/

	errors {
		/// Returned if an asset with a given name failed to load.
		Asset(name: String) {
			description("Failed to load asset")
			display("Failed to load asset with name {:?}", name)
		}

		/// Returned if a source could not retrieve something.
		Source {
			description("Failed to load bytes from source")
		}

		/// Returned if a format failed to load the asset data.
		Format(format: &'static str) {
			description("Format could not load asset")
			display("Format {:?} could not load asset", format)
		}
	}
}

pub trait Source: Send + Sync + 'static {
	fn load(&self, path: &str) -> Result<Vec<u8>>;
}

pub trait Asset: Send + Sync + 'static {
	type Data: Send + Sync + 'static;
}

pub trait Format<A: Asset> {
	type Options: Clone + Send + Sync + 'static;

	fn import(&self, bytes: Vec<u8>, options: Self::Options) -> Result<A::Data>;

	fn load(
		&self,
		name: String,
		source: Arc<Source>,
		options: Self::Options,
	) -> Result<A::Data> {
		let b = source.load(&name).chain_err(|| ErrorKind::Source)?;
		Ok(self.import(b, options)?)
	}
}

#[derive(Debug)]
pub struct Directory {
	loc: PathBuf,
}

impl Directory {
	pub fn new<P: Into<PathBuf>>(loc: P) -> Self {
		Self { loc: loc.into() }
	}

	pub fn path(&self, s_path: &str) -> PathBuf {
		let mut path = self.loc.clone();
		path.push(s_path);
		path
	}
}

impl Source for Directory {
	fn load(&self, path: &str) -> Result<Vec<u8>> {
		use std::io::Read;

		let path = self.path(path);

		let mut v = Vec::new();
		let mut file = File::open(&path)
			.chain_err(|| format!("Failed to open file {:?}", path))
			.chain_err(|| ErrorKind::Source)?;

		file.read_to_end(&mut v)
			.chain_err(|| format!("Failed to read file {:?}", path))
			.chain_err(|| ErrorKind::Source)?;

		Ok(v)
	}
}
