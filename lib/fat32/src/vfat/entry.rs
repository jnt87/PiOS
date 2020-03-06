use crate::traits;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    File(File<HANDLE>),
    Dir(Dir<HANDLE>),
}

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match &self {
            Entry::File(x) => &x.name,
            Entry::Dir(x) => &x.name,
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        use crate::traits::Metadata;
        match &self {
            Entry::File(x) => &x.metadata,
            Entry::Dir(x) => &x.metadata,
        }
    }

    fn as_file(&self) -> Option<&File<HANDLE>> {
        //use crate::traits::File;
        match &self {
            &Entry::File(ref x) => Some(x),
            &Entry::Dir(_) => None,
        }
    }
    fn as_dir(&self) -> Option<&Dir<HANDLE>> {
        //use crate::traits::Dir;
        match &self {
            &Entry::Dir(ref x) => Some(x),
            &Entry::File(_) => None,
        }
    }
    fn into_file(self) -> Option<File<HANDLE>> {
        //use crate::traits::File;
        match self {
            Entry::File(x) => Some(x),
            Entry::Dir(_) => None,
        }
    }
    fn into_dir(self) -> Option<Dir<HANDLE>> {
        //use crate::traits::Dir;
        match self {
            Entry::Dir(x) => Some(x),
            Entry::File(_) => None,
        }
    }
    fn is_file(&self) -> bool {
        match &self {
            &Entry::File(_) => true,
            &Entry::Dir(_) => false,
        }
    }
    fn is_dir(&self) -> bool {
        match &self {
            &Entry::Dir(_) => true,
            &Entry::File(_) => false,
        }
    }
}
