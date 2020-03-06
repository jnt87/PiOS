use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

use core::mem;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    head: u8,
    sector_upper: u8,
    cylinder_lower: u8,
}

// implement Debug for CHS
impl fmt::Debug for CHS {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("Header", &self.head)
            .field("Sector", &self.sector_upper) //masks?
            .field("Cylinder", &self.cylinder_lower) //masks?
            .finish()
    }
}
const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    pub boot: u8,
    chs_start: CHS,
    pub partition_type: u8,
    chs_end: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

// implement Debug for PartitionEntry
impl fmt::Debug for PartitionEntry {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionEntry")
            .field("boot", &self.boot)
            .field("CHS start", &self.chs_start)
            .field(
                "Partition_type",
                &format_args!("{}", 
                match &self.partition_type {
                    0xB => { "FAT32" },
                    0xC => { "FAT32"},
                    _ => { "NO IDEA" },
                })
            )
            .field("CHS end", &self.chs_end)
            .field("relative sector", &self.relative_sector)
            .field("total sectors", &self.total_sectors)
            .finish()
    }
}
const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436], //instructions are words but its not 64???
    disk_ID: [u8; 10],
    pub partitions: [PartitionEntry; 4],
    valid_sig: u16,
}

// implemente Debug for MaterBootRecord
impl fmt::Debug for MasterBootRecord {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("disk_ID", &self.disk_ID)
            .field("partitions", &self.partitions)
            .field("signature", &self.valid_sig)
            .finish()
    }
}
const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut mbr_back = [0u8; 512];
        let size = device.read_sector(0, &mut mbr_back).unwrap();
        if size != 512 {
            return Err(Error::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof, 
                    "Incorrect MBR size was read"
                )));
        }
        let mbr = unsafe { mem::transmute::<_, MasterBootRecord>(mbr_back) };
        if mbr.valid_sig != 0xAA55 {
            return Err(Error::BadSignature);
        }

        for i in 0..4 {
            if !((mbr.partitions[i].boot == 0x00) | (mbr.partitions[i].boot == 0x80)) {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }
        Ok(mbr)
    }
}
