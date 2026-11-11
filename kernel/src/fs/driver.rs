use common::io_error::IOError;
use common::opened_resource::Resource;
use common::path::Path;

pub trait DiskDriver {
    fn create(&self, path: Path) -> Result<impl Resource, IOError>;
    fn open(&self, path: Path) -> Result<impl Resource, IOError>;
}
