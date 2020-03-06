use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;
use core::mem;
use shim::io::Write;
use shim::path::Component;
use alloc::string::String;

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};
use crate::vfat::{Metadata, Attributes, Time, Date, Timestamp};
use crate::vfat;

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    pub rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {
        let boot_record = MasterBootRecord::from(&mut device)?;
        let part = &boot_record.partitions[0];
        if !(part.partition_type == 0xB || part.partition_type == 0xC) {
            return Err(vfat::error::Error::Io(<io::Error>::new(io::ErrorKind::NotFound, "wrong part type")));
        }
        let ebpb = BiosParameterBlock::from(&mut device, part.relative_sector as u64)?;
        //println!("{:?}", ebpb);
        //panic!("AHHHH");
        if ebpb.sectors_per_fat == 0 {
            return Err(vfat::error::Error::Io(<io::Error>::new(io::ErrorKind::NotFound, "ebpb.sectors_per_fat are wrong")));

        }
        let bytes_per_sector = ebpb.bytes_per_sector;
        let sectors_per_cluster = ebpb.sectors_per_cluster;
        let sectors_per_fat = ebpb.sectors_per_fat;
        let fat_start_sector = ebpb.sectors_reserved as u64;// + part.relative_sector as u64; //relative
        let data_start_sector = fat_start_sector + (sectors_per_fat as u64 * ebpb.num_fats as u64 ); 
        let rootdir_cluster = Cluster::from(ebpb.root_dir_cluster);
        let device = CachedPartition::new(
            device,
            Partition {
                start: part.relative_sector as u64,
                num_sectors: part.total_sectors as u64,
                sector_size: bytes_per_sector as u64,
            },
        );
        let virtualfat = VFat {
            phantom: PhantomData,
            device: device,
            bytes_per_sector: bytes_per_sector,
            sectors_per_cluster: sectors_per_cluster,
            sectors_per_fat: sectors_per_fat as u32,
            fat_start_sector: fat_start_sector,
            data_start_sector: data_start_sector,
            rootdir_cluster: rootdir_cluster,
        }; 
        if virtualfat.sectors_per_fat == 0 {
            return Err(vfat::error::Error::Io(<io::Error>::new(io::ErrorKind::NotFound, "sectors_per_fat are wrong")));
        }
        
        Ok(HANDLE::new(virtualfat))
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    //    fn read_cluster(
    //        &mut self,
    //        cluster: Cluster,
    //        offset: usize,
    //        buf: &mut [u8]
    //    ) -> io::Result<usize>;
    pub fn read_cluster( &mut self, cluster: Cluster, offset: usize, mut buf: &mut [u8]) -> io::Result<usize> {
        let beginning = self.data_start_sector + (cluster.cluster_number() as u64 - 2) * self.sectors_per_cluster as u64;
        let mut bytes_read: usize = 0;
        loop {
            let index = (offset + bytes_read) as u64 / self.bytes_per_sector as u64;
            if index >= self.sectors_per_cluster as u64 {
                break;
            } else {
                let byte_offset = (offset + bytes_read) as usize - index as usize * self.bytes_per_sector as usize;
                let data = self.device.get(beginning + index)?;

                let bytes = buf.write(&data[byte_offset..])?;
                bytes_read = bytes_read + bytes;

                if buf.is_empty() {
                    break;
                }
            }
        }
        Ok(bytes_read)
    }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    //    fn read_chain(
    //        &mut self,
    //        start: Cluster,
    //        buf: &mut Vec<u8>
    //    ) -> io::Result<usize>;
    pub fn read_chain( &mut self, start: Cluster, buf: &mut Vec<u8> ) -> io::Result<usize> {
        let mut cluster = start;
        let mut bytes_read = 0;
        loop {
            let entry = self.fat_entry(cluster)?.status();

            match entry {
                Status::Data(next_cluster) => {
                    let size = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
                    buf.reserve(size);
                    let old_len = buf.len();
                    let len_needed = old_len + size;
                    
                    //resize 
                    unsafe { 
                        buf.set_len(len_needed)
                    }
                    bytes_read = self.read_cluster(cluster, 0, &mut buf[old_len..])?;
                    unsafe {
                        buf.set_len(old_len + bytes_read);
                    }

                    cluster = next_cluster;
                },
                Status::Eoc(_) => {
                    let size = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
                    buf.reserve(size);
                    let old_len = buf.len();
                    let len_needed = old_len + size;
                    
                    //resize 
                    unsafe { 
                        buf.set_len(len_needed)
                    }
                    bytes_read = self.read_cluster(cluster, 0, &mut buf[old_len..])?;
                    unsafe {
                        buf.set_len(old_len + bytes_read);
                    }
                    break;
                },
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid entry")),
            }
        }
        Ok(bytes_read)
    }
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
    //
    //
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let sector = self.fat_start_sector as u64 + (cluster.cluster_number() as u64 * 4) / (self.bytes_per_sector as u64);
        let index = (cluster.cluster_number() as u64 * 4) % self.bytes_per_sector as u64;
        let data = self.device.get(sector)?;
        //let addr = sector * self.bytes_per_sector as u64 + index;
        let virtualfat = unsafe { &data[(index as usize)..(index as usize+4)].cast()[0] };
        Ok(virtualfat)
    }



    pub fn find_cluster(&mut self, start: Cluster, offset: usize) -> io::Result<(Cluster, usize)>
    {
        let size = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        let cluster_index = offset / size;
        let mut cluster = start;

        for i in 0..cluster_index {
            let fat_entry = self.fat_entry(cluster)?.status();
            match fat_entry {
                Status::Data(next) => {
                    cluster = next;
                },
                Status::Eoc(_) => {
                    if i + 1 != cluster_index {
                        return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "incorrect size"));
                    }

                    cluster = Cluster::from(0xFFFFFFFF);
                }, 
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "bad cluster entry")),
            } 

        }
        Ok((cluster, cluster_index * size))
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let root_dir = self.lock(|vfat| vfat.rootdir_cluster);
        let cluster_high = (root_dir.cluster_number() >> 16) as u16;
        let cluster_low = (root_dir.cluster_number() & 0xFFFF) as u16;
        let metadata = Metadata {
           attr: Attributes(0x3),
            _win: 0x00,
            create_time_mantissa: 0x00,
            create_time: Time(0x0000),
            create_date: Date(0x0000),
            access_date: Date(0x0000),
            cluster_high: cluster_high,
            mod_time: Time(0x0000),
            mod_date: Date(0x0000),
            cluster_low: cluster_low,
            size: 0,
        };
        let mut dir = Entry::Dir(Dir {
            vfat: self.clone(), 
            first_cluster: root_dir,
            name: String::from("/"),
            metadata: metadata,
        });
        for x in path.as_ref().components() {
            match x {
                //used here suppose to be in shell - pathbuf.pop() to get to parent
                /*Component::ParentDir => {
                    use crate::traits::Entry;
                    dir = dir.into_dir().unwrap().find("..")?;
                },*/
                Component::Normal(name) => {
                    use crate::traits::Entry;
                    dir = dir.into_dir().unwrap().find(name)?;
                }
                _ => (),
            }
        }
        Ok(dir)

        // I believe this is where you would but create_file, create_dir, rename, remove, etc
        // for a write enabled filesystem
    }
}
