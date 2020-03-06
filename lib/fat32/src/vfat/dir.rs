use alloc::string::String;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;
use core::char::decode_utf16;

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle}; 

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub first_cluster: Cluster,
    pub name: String,
    pub metadata: Metadata,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    filename: [u8; 8],
    ext: [u8; 3],
    pub attr: Attributes, //u8
    _win: u8, 
    create_time_mantissa: u8,
    create_time: Time, //u16
    create_date: Date, //u16
    access_date: Date, //u16
    cluster_high: u16,
    mod_time: Time, //u16
    mod_date: Date, //u16
    cluster_low: u16,
    file_size: u32,
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    seq_num: u8,
    file_name1: [u16; 5],
    pub attr: u8, 
    type_fat_entry: u8,
    checksum: u8,
    file_name2: [u16; 6],
    _zeros: [u8; 2],
    file_name3: [u16; 2],
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    info: u8,
    _unknown: [u8; 10],
    pub attr: u8,
    _unknown2: [u8; 20],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl VFatRegularDirEntry {
    pub fn filename(&self) -> String {
        let mut term_index = 0; //termination index of string
        let name_clone = self.filename.clone();
        for i in 0..name_clone.len() {
            if name_clone[i] == 0x00 || name_clone[i] == 0x20 {
                break;
            }
            term_index = term_index + 1;
        }
        let mut filename = String::from_utf8(name_clone[..term_index].to_vec()).unwrap();
        let extension_clone = self.ext.clone();
        if self.attr.0 & 0x0010 == 0 {  //is not directory
            let extension = String::from_utf8(extension_clone.to_vec()).unwrap();
            if !extension.is_empty() && extension.as_str()[..1] != *" " {
                filename.push('.');
                
                for character in extension.chars() {
                    if character != ' ' {
                        filename.push(character);
                    } else {
                        break; 
                    }
                }
                return filename;
            }
        }
        filename
    }
}

impl VFatLfnDirEntry {
    pub fn position(&self) -> usize {
        let seq_number = &self.seq_num & 0x1F & 0x1F;
        assert!(seq_number != 0);
        seq_number as usize
    }

    pub fn last(&self) -> bool {
        &self.seq_num & 0x30 == 0
    }

    pub fn build_filename(&self, dir: &mut Vec<u16>) {
        let start = dir.len();
        dir.extend_from_slice(&self.file_name1);
        dir.extend_from_slice(&self.file_name2);
        dir.extend_from_slice(&self.file_name3);
        
        for index in start..dir.len() {
            if dir[index] == 0x00FF || dir[index] == 0x0000 {
                dir.resize(index, 0);
                return;
            }
        }
    }
}

impl VFatUnknownDirEntry {
    pub fn is_longfilename(&self) -> bool {
        self.attr == 0x0F
    }

    pub fn is_last(&self) -> bool {
        self.info == 0x00
    }

    pub fn empty(&self) -> bool {
        self.info == 0xE5
    }
}

pub struct DirectoryIter<HANDLE: VFatHandle> {
    vfat: HANDLE,
    data: Vec<VFatDirEntry>,
    index: usize,
}




impl<HANDLE: VFatHandle> DirectoryIter<HANDLE> {
    fn long_filename(longname: &mut Vec<&VFatLfnDirEntry>) -> String {
        longname.sort_by_key(|i| i.position());

        let mut name_memback: Vec<u16> = Vec::with_capacity(13 * longname.len());
        for entry in longname.iter() {
            entry.build_filename(&mut name_memback);
        }
        
        decode_utf16(name_memback.clone())
            .map(|r| r.unwrap_or('?')).collect::<String>()
    }




    fn create_entry(&self, longfilename: &mut Vec<&VFatLfnDirEntry>, entry: VFatRegularDirEntry) -> Entry<HANDLE> {
        let name = if longfilename.is_empty() {
            entry.filename()
        } else {
            DirectoryIter::<HANDLE>::long_filename(longfilename)
        };

        let metadata = Metadata::new(
            entry.attr,
            entry._win,
            entry.create_time_mantissa,
            entry.create_time,
            entry.create_date,
            entry.access_date,
            entry.cluster_high,
            entry.mod_time,
            entry.mod_date,
            entry.cluster_low,
            entry.file_size,
        );

        let cluster_start = (entry.cluster_high as u32) << 16 | (entry.cluster_low as u32);
        //magic happening
        if entry.attr.0 & 0x10 != 0 { //is directory
            Entry::Dir(Dir { 
                vfat: self.vfat.clone(), 
                first_cluster: Cluster::from(cluster_start), 
                name: name, 
                metadata: metadata,
            })
            
        } else {
            //metadata
            Entry::File(File { 
                vfat: self.vfat.clone(),
                start: Cluster::from(cluster_start),
                size: entry.file_size,
                pointer: 0,
                current_cluster: Cluster::from(cluster_start), //CHECK THIS
                current_cluster_start: 0,
                name: name,
                metadata: metadata,
            })
        }
    }
}



impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        use traits::{Dir, Entry};

        let name = name.as_ref().to_str().unwrap();
        for entry in self.entries()? {
            if entry.name().eq_ignore_ascii_case(name) {
                return Ok(entry);
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Entry not found"))
        /*Ok(self.entries()?.find(|item| {
            item.name().eq_ignore_ascii_case(name)
        }).unwrap())*/
    }
}


impl<HANDLE: VFatHandle> Iterator for DirectoryIter<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut longname: Vec<&VFatLfnDirEntry> = Vec::with_capacity(20);

        for x in self.index..self.data.len() { //vec vfatdirentry
            let entry = &self.data[x];
            let entry_unknown = unsafe { entry.unknown };
            if entry_unknown.is_last() {
                break;
            }
            if entry_unknown.empty() {
                continue;
            }
            if entry_unknown.is_longfilename() {
                longname.push(unsafe { &entry.long_filename });
            } else {
                self.index = x + 1;
                return Some(self.create_entry(&mut longname, unsafe { entry.regular }));
            }
        }
        self.index = self.data.len();
        None
    }
}
impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>; //pretty sure this needs to be here
    type Iter = DirectoryIter<HANDLE>;

    fn entries(&self) -> io::Result<Self::Iter> { //all kinds of screwy
        let mut data = Vec::new();
        self.vfat.lock(|vfat| vfat.read_chain(self.first_cluster, &mut data))?;
        let all_entries = DirectoryIter {
            vfat: self.vfat.clone(), //added clone
            data: unsafe { data.cast() },
            index: 0,
        };
        Ok(all_entries)
    }
    
}
