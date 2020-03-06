use core::fmt;
use core::mem;
use shim::io;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    // Fill me in.
    jmp_short_xx_nop: [u8; 3],
    oem_ident: [u8; 8], //not u64??
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub sectors_reserved: u16,
    pub num_fats: u8,
    MAX_ENTRIES: u16,
    total_sectors: u16,
    media_descriptor_type: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    number_hidden_sectors: u32,
    total_logic_sectors: u32,
    //extended block offset = 36 bytes
    pub sectors_per_fat: u32,
    flags: u16,
    fat_version: u16,
    pub root_dir_cluster: u32,
    fsinfo_structure_sec_location: u16,
    backup_boot_sec_location: u16,
    _reserved1: [u8; 12],
    drive_num: u8,
    _reserved2: u8,
    signature: u8, // should be 0x28 or 0x29
    volume_id_serial: u32,
    volume_label: [u8; 11],
    fs_string: [u8; 8],
    _bootcode: [u8; 420],
    boot_part_sig: u16, //should be 0xAA55 for bootable
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut back_end = [0u8; 512];
        let mut size = device.read_sector(sector, &mut back_end)?;
        if size != 512 { 
            Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof, 
                "bad EBPB signature",
            )))
        } else {
            let geometry = unsafe { mem::transmute::<_,BiosParameterBlock>(back_end) };
            if geometry.boot_part_sig == 0xAA55 {
                if geometry.sectors_per_fat == 0 {
                    //return Err(Error::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "sectors per fat are 0 in BPB")));
                }
                Ok(geometry)
            } else {
                Err(Error::BadSignature)
            }
        }
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //add stuff here if you want it printed out later
        f.debug_struct("BiosParameterBlock").finish()
    }
}
