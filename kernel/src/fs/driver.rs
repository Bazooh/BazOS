use common::io_error::IOError;
use common::opened_resource::OpenedResource;
use common::path::Path;

pub trait DiskDriver {
    fn create(&self, path: Path) -> Result<impl OpenedResource, IOError>;
    fn open(&self, path: Path) -> Result<impl OpenedResource, IOError>;
}
