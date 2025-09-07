use std::{
	path::{Path, PathBuf},
	str::FromStr,
	sync::{Arc, RwLock},
};

use bm_version::VersionKey;
use ironworks::{
	Ironworks,
	excel::Excel,
	sqpack::{Install, SqPack},
};
use tokio::sync::watch;

use super::error::{Error, Result};

pub struct Data {
	channel: watch::Sender<Vec<VersionKey>>,
	version: Arc<RwLock<Option<Arc<Version>>>>,
	version_key: VersionKey,
	game_dir: PathBuf,
}

impl Data {
	pub fn new(game_dir: PathBuf) -> Self {
		let (sender, _receiver) = watch::channel(vec![]);

		Data {
			channel: sender,
			version: Arc::new(RwLock::new(None)),
			// Use a constant version key
			version_key: VersionKey::from_str("0000000000000001").unwrap(),
			game_dir,
		}
	}

	pub fn ready(&self) -> bool {
		self.version.read().expect("poisoned").is_some()
	}

	pub fn subscribe(&self) -> watch::Receiver<Vec<VersionKey>> {
		self.channel.subscribe()
	}

	/// Initialize with a single version using filesystem data
	pub fn initialize(&self) -> Result<()> {
		let version = Version::new(&self.game_dir)?;

		*self.version.write().expect("poisoned") = Some(Arc::new(version));

		// Broadcast the version list
		self.broadcast_version_list();

		Ok(())
	}

	pub fn version_key(&self) -> VersionKey {
		self.version_key
	}

	pub fn version(&self, version: VersionKey) -> Result<Arc<Version>> {
		self.version
			.read()
			.expect("poisoned")
			.clone()
			.ok_or_else(|| Error::UnknownVersion(version))
	}

	fn broadcast_version_list(&self) {
		let keys = vec![self.version_key];

		self.channel.send_if_modified(|value| {
			if &keys != value {
				*value = keys;
				return true;
			}
			false
		});
	}
}

pub struct Version {
	ironworks: Arc<Ironworks>,
	excel: Arc<Excel>,
}

impl Version {
	fn new(game_dir: &Path) -> Result<Self> {
		let install = Install::at(game_dir);
		let ironworks = Arc::new(Ironworks::new().with_resource(SqPack::new(install)));
		let excel = Arc::new(Excel::new(ironworks.clone()));
		Ok(Self { ironworks, excel })
	}

	pub fn ironworks(&self) -> Arc<Ironworks> {
		self.ironworks.clone()
	}

	pub fn excel(&self) -> Arc<Excel> {
		self.excel.clone()
	}
}
