use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use conquer_once::noblock::OnceCell;

use crate::{
    fs::{
        device::BlockDevice,
        driver::{DiskDriver, IOError},
        file::File,
        path::Path,
    },
    io::{
        device::BLOCK_DEVICE,
        disk::{
            BLOCK_SIZE, DATA_BITMAP_OFFSET, DATA_OFFSET, INODE_BITMAP_OFFSET, INODE_OFFSET,
            bitmap::VirtDiskBitmap,
            file::VirtDiskFile,
            header::VirtDiskHeader,
            inode::{VirtDiskINode, u8_array_mut_view},
        },
    },
};

#[derive(Debug, Clone, Copy)]
pub struct BlockId(usize);

impl BlockId {
    pub fn is_none(&self) -> bool {
        return self.0 == 0;
    }

    pub fn inode_bitmap(index: usize) -> Self {
        BlockId(INODE_BITMAP_OFFSET + index)
    }

    pub fn data_bitmap(index: usize) -> Self {
        BlockId(DATA_BITMAP_OFFSET + index)
    }

    pub fn inode(index: usize) -> Self {
        BlockId(INODE_OFFSET + index)
    }

    pub fn data(index: usize) -> Self {
        BlockId(DATA_OFFSET + index)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    pub fn raw(index: usize) -> Self {
        BlockId(index)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct INodeIndex(usize);

impl INodeIndex {
    pub fn new(index: usize) -> Self {
        INodeIndex(index)
    }

    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub fn to_block_id(&self) -> BlockId {
        BlockId::inode(self.0)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

pub struct VirtDiskDriver {
    header: VirtDiskHeader,
    root_dir_index: INodeIndex,
}

pub static DISK_DRIVER: OnceCell<VirtDiskDriver> = OnceCell::uninit();

impl VirtDiskDriver {
    fn flip_free_inode_index(&self) -> Option<INodeIndex> {
        let mut buffer = [0u8; BLOCK_SIZE];
        for bitmap_index in self.header.inodes_bitmap_indexes() {
            let mut device = BLOCK_DEVICE.lock();
            device.read(bitmap_index.0, &mut buffer);
            let bitmap = VirtDiskBitmap::from_buffer(&buffer);
            if let Some(index) = bitmap.find_zero() {
                buffer[index / u64::BITS as usize] |= 1 << (index % u64::BITS as usize); // TODO: verify direction
                device.write(bitmap_index.0, &buffer);
                return Some(INodeIndex(
                    (bitmap_index.0 - INODE_BITMAP_OFFSET) * 8 * BLOCK_SIZE + index,
                ));
            }
        }
        None
    }

    pub fn get_inode(&self, index: INodeIndex) -> Result<VirtDiskINode, String> {
        let mut buffer = [0usize; BLOCK_SIZE / 8];
        let view = u8_array_mut_view(&mut buffer);
        BLOCK_DEVICE
            .lock()
            .read(index.to_block_id().as_usize(), view);
        VirtDiskINode::from_buffer(buffer)
    }

    fn add_file(
        &self,
        name: &str,
        dir_inode_index: INodeIndex,
        file_inode_index: INodeIndex,
    ) -> Result<(), IOError> {
        // TODO: not thread safe
        match self.get_inode(dir_inode_index).unwrap() {
            VirtDiskINode::Directory(mut dir) => {
                let result = dir
                    .add_file(name, file_inode_index)
                    .ok_or(IOError::DirectoryFull);
                BLOCK_DEVICE
                    .lock()
                    .write(dir_inode_index.to_block_id().as_usize(), dir.buffer());
                result
            }
            _ => Err(IOError::NotADirectory(String::from(name))),
        }
    }
}

impl DiskDriver for VirtDiskDriver {
    fn create(&self, path: Path) -> Result<impl File, IOError> {
        let inode_index = self.flip_free_inode_index().ok_or(IOError::NoSpace)?;

        let mut parts: Vec<&str> = path.split();
        let file_name = parts.pop().ok_or(IOError::InvalidPath)?;

        let dir_inode_index = self.root_dir_index;
        // for part in parts {
        //     match inode_dir.get(part, self) {
        //         Some(inode) => match inode {
        //             VirtDiskINode::File(..) => {
        //                 return Err(FileCreationError::NotADirectory(String::from(part)));
        //             }
        //             VirtDiskINode::Directory(dir_node) => {
        //                 inode_dir = dir_node;
        //             }
        //         },
        //         None => todo!(),
        //     };
        // }

        self.add_file(file_name, dir_inode_index, inode_index)?;

        Ok(VirtDiskFile::new(String::from(file_name), inode_index))
    }

    fn open(&self, path: Path) -> Result<impl File, IOError> {
        let mut parts: Vec<&str> = path.split();
        let file_name = parts.pop().ok_or(IOError::InvalidPath)?;

        let dir_inode_index = self.root_dir_index;
        let VirtDiskINode::Directory(dir_inode) = self.get_inode(dir_inode_index).unwrap() else {
            panic!("Root directory is not a directory");
        };

        match dir_inode.get(file_name) {
            Some((inode_index, inode)) => match inode {
                VirtDiskINode::File(..) => {
                    Ok(VirtDiskFile::new(String::from(file_name), inode_index))
                }
                VirtDiskINode::Directory(..) => Err(IOError::CannotOpenADirectory),
            },
            None => Err(IOError::DoesNotExist),
        }
    }
}

pub fn init() {
    let mut buffer = [0u8; BLOCK_SIZE];
    BLOCK_DEVICE.lock().read(0, &mut buffer);
    let header = VirtDiskHeader::from_buffer(buffer).expect("Disk not compatible");
    DISK_DRIVER
        .try_init_once(|| VirtDiskDriver {
            header,
            root_dir_index: INodeIndex(0),
        })
        .expect("Disk driver already initialized");
}
