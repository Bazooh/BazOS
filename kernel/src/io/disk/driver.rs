use crate::io::disk::NUMBER_BLOCKS;
use crate::{
    fs::{device::BlockDevice, driver::DiskDriver},
    io::{
        device::BLOCK_DEVICE,
        disk::{
            BLOCK_SIZE, DATA_BITMAP_OFFSET, DATA_OFFSET, INODE_BITMAP_OFFSET, INODE_OFFSET,
            bitmap::VirtDiskBitmap, file::VirtDiskFile, header::VirtDiskHeader,
            inode::VirtDiskINode,
        },
    },
};
use alloc::vec::Vec;
use common::io_error::IOError;
use common::opened_resource::OpenedResource;
use common::path::Path;
use conquer_once::noblock::OnceCell;

#[derive(Debug, Clone, Copy)]
pub enum BlockId {
    Header,
    INode(u64),
    INodeBitmap(u64),
    Data(u64),
    DataBitmap(u64),
}

impl BlockId {
    pub fn header() -> BlockId {
        BlockId::Header
    }

    pub fn data_bitmap(index: u64) -> BlockId {
        BlockId::DataBitmap(index + DATA_BITMAP_OFFSET)
    }

    pub fn inode_bitmap(index: u64) -> BlockId {
        BlockId::INodeBitmap(index + INODE_BITMAP_OFFSET)
    }

    pub fn inode_bitmaps() -> impl Iterator<Item = BlockId> {
        (INODE_BITMAP_OFFSET..INODE_OFFSET).map(|block_id| BlockId::raw(block_id).unwrap())
    }

    pub fn inode(index: u64) -> BlockId {
        BlockId::INode(index + INODE_OFFSET)
    }

    pub fn data(index: u64) -> BlockId {
        BlockId::Data(index + DATA_OFFSET)
    }

    pub fn block_id(&self) -> u64 {
        match self {
            BlockId::Header => 0,
            BlockId::DataBitmap(block_id) => *block_id,
            BlockId::INodeBitmap(block_id) => *block_id,
            BlockId::INode(block_id) => *block_id,
            BlockId::Data(block_id) => *block_id,
        }
    }

    pub fn index(&self) -> u64 {
        match self {
            BlockId::Header => 0,
            BlockId::DataBitmap(block_id) => *block_id - DATA_BITMAP_OFFSET,
            BlockId::INodeBitmap(block_id) => *block_id - INODE_BITMAP_OFFSET,
            BlockId::INode(block_id) => *block_id - INODE_OFFSET,
            BlockId::Data(block_id) => *block_id - DATA_OFFSET,
        }
    }

    pub fn raw(block_id: u64) -> Option<Self> {
        match block_id {
            DATA_BITMAP_OFFSET..INODE_BITMAP_OFFSET => Some(BlockId::DataBitmap(block_id)),
            INODE_BITMAP_OFFSET..INODE_OFFSET => Some(BlockId::INodeBitmap(block_id)),
            INODE_OFFSET..DATA_OFFSET => Some(BlockId::INode(block_id)),
            DATA_OFFSET..NUMBER_BLOCKS => Some(BlockId::Data(block_id)),
            _ => None,
        }
    }
}

pub struct VirtDiskDriver {
    header: VirtDiskHeader,
    root_dir: BlockId,
}

pub static DISK_DRIVER: OnceCell<VirtDiskDriver> = OnceCell::uninit();

impl VirtDiskDriver {
    fn flip_free_inode_index(&self) -> Option<BlockId> {
        let mut buffer = [0u8; BLOCK_SIZE];
        for inode_bitmap in BlockId::inode_bitmaps() {
            let mut device = BLOCK_DEVICE.lock();
            device.read(inode_bitmap, &mut buffer);
            let bitmap = VirtDiskBitmap::from_buffer(&buffer);
            if let Some(index) = bitmap.find_zero() {
                buffer[index / u64::BITS as usize] |= 1 << (index % u64::BITS as usize); // TODO: verify direction
                device.write(inode_bitmap, &buffer);
                return Some(BlockId::inode(
                    inode_bitmap.index() * 8 * BLOCK_SIZE as u64 + index as u64,
                ));
            }
        }
        None
    }

    fn add_file(&self, name: &str, dir_inode: BlockId, file_inode: BlockId) -> Result<(), IOError> {
        todo!()
    }
}

impl DiskDriver for VirtDiskDriver {
    fn create(&self, path: Path) -> Result<impl OpenedResource, IOError> {
        let block_id = self
            .flip_free_inode_index()
            .ok_or(IOError::NotEnoughSpace)?;

        let mut parts: Vec<&str> = path.split();
        let file_name = parts.pop().ok_or(IOError::InvalidPath)?;

        Ok(VirtDiskFile::new(todo!()).unwrap())
    }

    fn open(&self, path: Path) -> Result<impl OpenedResource, IOError> {
        let mut parts: Vec<&str> = path.split();
        let file_name = parts.pop().ok_or(IOError::InvalidPath)?;

        let VirtDiskINode::Directory(root) = VirtDiskINode::read(self.root_dir).unwrap() else {
            panic!("Root directory is not a directory");
        };

        match root.read(file_name) {
            Some(inode) => match inode {
                VirtDiskINode::File(inode) => {
                    VirtDiskFile::new(inode).map_err(|e| IOError::AlreadyOpened)
                }
                VirtDiskINode::Directory(..) => Err(IOError::CannotOpenADirectory),
            },
            None => Err(IOError::DoesNotExist(path)),
        }
    }
}

pub fn init() {
    let mut buffer = [0u8; BLOCK_SIZE];
    BLOCK_DEVICE.lock().read(BlockId::header(), &mut buffer);
    let header = VirtDiskHeader::from_buffer(buffer).expect("Disk not compatible");
    DISK_DRIVER
        .try_init_once(|| VirtDiskDriver {
            header,
            root_dir: BlockId::inode(0),
        })
        .expect("Disk driver already initialized");
}
