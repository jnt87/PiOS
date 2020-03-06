use alloc::string::String;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle, VFat};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub start: Cluster,
    pub size: u32,
    pub pointer: u64,
    pub current_cluster: Cluster,
    pub current_cluster_start: usize,
    pub name: String,
    pub metadata: Metadata,
}

impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn size(&self) -> u64 {
        self.size as u64
    }
}
impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut num_bytes_read: usize = 0;
        let max = if (self.size as usize - self.pointer as usize) < buf.len() { 
            self.size as usize - self.pointer as usize
        } else { 
            buf.len() 
        };
        while self.pointer < self.size as u64 {
            let bytes = self.vfat.lock(|vfat| vfat.read_cluster(
                self.current_cluster, 
                self.pointer as usize - self.current_cluster_start,
                &mut buf[num_bytes_read..max]))?;
            if bytes == 0 {
                break;
            }

            num_bytes_read = num_bytes_read + bytes;

            self.pointer = self.pointer + bytes as u64;

            let (cluster, offset) = self.vfat.lock(|vfat| vfat.find_cluster( //removed borrow_mut() after vfat
                self.current_cluster,
                self.pointer as usize - self.current_cluster_start))?;
            self.current_cluster = cluster;
            self.current_cluster_start = self.current_cluster_start + offset;
        }
        Ok(num_bytes_read)
    }

}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("Write")
    }
    
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        match _pos {
            SeekFrom::Start(offset) => {
                if offset >= self.size as u64 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "OOB")) //ioerr! shim::io::{Error, ErrorKind}
                } else {
                    self.pointer = offset;
                    Ok(offset)
                }
            },
            SeekFrom::Current(offset) => {
                if (offset + self.pointer as i64) >= (self.size as i64) || (offset + self.pointer as i64) < 0 { 
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "OOB"))
                } else {
                    let pointer = self.pointer + offset as u64;
                    self.pointer = pointer;
                    Ok(pointer)
                }
            },
            SeekFrom::End(offset) => {
                if offset >= 0 || offset + (self.size as i64) < 0 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "OOB"))
                } else {
                    let pointer = (self.size as i64 + offset) as u64;
                    self.pointer = pointer;
                    Ok(pointer)
                }
            }
        }
    }
}
