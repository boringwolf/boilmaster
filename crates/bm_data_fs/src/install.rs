use std::{
	fs,
	io::{self, Seek},
	path::{Path, PathBuf},
};

use super::error::Result;
use crate::take_seekable::{TakeSeekable, TakeSeekableExt};
use ironworks::{
	Error, ErrorValue,
	sqpack::{Location, Resource},
};

/// SqPack resource for reading game data from an on-disk FFXIV installation.
#[derive(Debug)]
pub struct Install {
	path: PathBuf,
	version_path: PathBuf,
	repositories: Vec<Option<String>>,
}

impl Install {
	/// Configure a resource instance with an installation of FFXIV at the specified path.
	pub fn at(path: &Path) -> Self {
		let sqpack_path = path.join("sqpack");
		let repositories = find_repositories(&sqpack_path);
		let version_path = path.join("ffxivgame.ver");

		Self {
			path: sqpack_path,
			version_path,
			repositories,
		}
	}

	fn build_file_path(
		&self,
		repository: u8,
		category: u8,
		chunk: u8,
		extension: &str,
	) -> Result<PathBuf, Error> {
		let platform = "win32";
		let file_name = format!("{category:02x}{repository:02x}{chunk:02x}.{platform}.{extension}");
		let file_path = self.path.join(
			[self.get_repository_name(repository)?, &file_name]
				.iter()
				.collect::<PathBuf>(),
		);

		Ok(file_path)
	}

	fn get_repository_name(&self, repository: u8) -> Result<&String, Error> {
		self.repositories
			.get(usize::from(repository))
			.and_then(|option| option.as_ref())
			.ok_or_else(|| Error::NotFound(ErrorValue::Other(format!("repository {repository}"))))
	}
}

impl Resource for Install {
	fn version(&self, _repository: u8) -> Result<String, Error> {
		let version =
			fs::read_to_string(&self.version_path).map_err(|error| match error.kind() {
				io::ErrorKind::NotFound => Error::NotFound(ErrorValue::Other(format!(
					"file path {}",
					self.version_path.display()
				))),
				_ => Error::Resource(error.into()),
			})?;
		Ok(version.trim().to_string())
	}

	type Index = io::Cursor<Vec<u8>>;
	fn index(&self, repository: u8, category: u8, chunk: u8) -> Result<Self::Index, Error> {
		read_index(self.build_file_path(repository, category, chunk, "index")?)
	}

	type Index2 = io::Cursor<Vec<u8>>;
	fn index2(&self, repository: u8, category: u8, chunk: u8) -> Result<Self::Index2, Error> {
		read_index(self.build_file_path(repository, category, chunk, "index2")?)
	}

	type File = TakeSeekable<io::BufReader<fs::File>>;
	fn file(&self, repository: u8, category: u8, location: Location) -> Result<Self::File, Error> {
		let path = self.build_file_path(
			repository,
			category,
			location.chunk(),
			&format!("dat{}", location.data_file()),
		)?;
		let mut file = io::BufReader::new(fs::File::open(path)?);

		let offset = u64::from(location.offset());
		// Resolve the size early in case we need to seek to find the end. Using
		// longhand here so I can shortcut seek failures.
		let size = match location.size() {
			Some(size) => u64::from(size),
			None => file.seek(io::SeekFrom::End(0))? - offset,
		};

		file.seek(io::SeekFrom::Start(offset))?;

		Ok(file.take_seekable(size)?)
	}
}

fn find_repositories(path: &Path) -> Vec<Option<String>> {
	(0..=9)
		.map(|index| {
			let name = match index {
				0 => "ffxiv".into(),
				other => format!("ex{other}"),
			};

			path.join(&name).exists().then_some(name)
		})
		.collect()
}

fn read_index(path: PathBuf) -> Result<io::Cursor<Vec<u8>>, Error> {
	// Read the entire index into memory before returning - we typically need
	// the full dataset anyway, and working directly on a File causes significant
	// slowdowns due to IO syscalls.
	let buffer = fs::read(&path).map_err(|error| match error.kind() {
		io::ErrorKind::NotFound => {
			Error::NotFound(ErrorValue::Other(format!("file path {path:?}")))
		}
		_ => Error::Resource(error.into()),
	})?;
	Ok(io::Cursor::new(buffer))
}
