use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(pub u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(pub u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attr: Attributes,
    pub _win: u8,
    pub create_time_mantissa: u8,
    pub create_time: Time,
    pub create_date: Date,
    pub access_date: Date,
    pub cluster_high: u16,
    pub mod_time: Time,
    pub mod_date: Date,
    pub cluster_low: u16,
    pub size: u32,
}
impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        (self.date.0 >> 9) as usize + 1980
    }

    fn month(&self) -> u8 {
        ((self.date.0 >> 5) & 0x000F) as u8
    }

    fn day(&self) -> u8 {
        (self.date.0 & 0x001F) as u8
    } 

    fn hour(&self) -> u8 {
        (self.time.0 >> 11) as u8
    }

    fn minute(&self) -> u8 {
        ((self.time.0 >> 5) & 0x003F) as u8
    }

    fn second(&self) -> u8 {
        (self.time.0 & 0x001F) as u8 * 2
    }
}

impl Metadata {
    pub fn new(attr: Attributes, _win: u8, create_time_mantissa: u8, create_time: Time, create_date: Date, access_date: Date, cluster_high: u16, mod_time: Time, mod_date: Date, cluster_low: u16, size: u32) -> Metadata {
        Metadata { 
            attr,
            _win,
            create_time_mantissa,
            create_time,
            create_date,
            access_date,
            cluster_high,
            mod_time,
            mod_date,
            cluster_low,
            size,
        }
    }
    pub fn hidden(&self) -> bool {
        (self.attr.0 & 0x0002) != 0
    }

}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;
    // first handle permissions
    fn read_only(&self) -> bool {
       (self.attr.0 & 0x0001) != 0
    }

    fn hidden(&self) -> bool {
        (self.attr.0 & 0x0002) != 0
    }

    fn accessed(&self) -> self::Timestamp {
        Timestamp {
            time: Time(0),
            date: self.access_date,
        }
    }

    fn modified(&self) -> self::Timestamp {
        Timestamp {
            time: self.mod_time,
            date: self.mod_date,
        }
    }

    fn created(&self) -> self::Timestamp {
        Timestamp {
            time: self.create_time,
            date: self.create_date,
        }
    }

}

impl traits::Metadata for &Metadata {
    type Timestamp = Timestamp;
    // first handle permissions
    fn read_only(&self) -> bool {
       (self.attr.0 & 0x0001) != 0
    }

    fn hidden(&self) -> bool {
        (self.attr.0 & 0x0002) != 0
    }

    fn accessed(&self) -> self::Timestamp {
        Timestamp {
            time: Time(0),
            date: self.access_date,
        }
    }

    fn modified(&self) -> self::Timestamp {
        Timestamp {
            time: self.mod_time,
            date: self.mod_date,
        }
    }

    fn created(&self) -> self::Timestamp {
        Timestamp {
            time: self.create_time,
            date: self.create_date,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use traits::Timestamp;
        write!(f, "(MDY: {}-{}-{} {}:{:02}:{:02}", self.month(), self.day(), self.year(), self.hour(), self.minute(), self.second())
    }
}


impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use traits::Metadata;
        write!(f, "ro={} created={} accessed={} modified={}", &self.read_only(), &self.created(), &self.accessed(), &self.modified())
    }
}
// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
