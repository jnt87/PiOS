use shim::io;
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;

use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::ALLOCATOR;
use crate::FILESYSTEM;
use fat32::traits::Metadata;
use alloc::string::String;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) {
    let mut exit = false;
    let mut working_dir = PathBuf::from("/");
    loop {
        if exit == true  { break; }
        let mut buf_mem = [0u8; 512];
        let mut buffer = StackVec::new(&mut buf_mem);
        //loop {
            let mut x = CONSOLE.lock();
            kprint!("\r{}", prefix);
        //    if x.inner().has_byte() { break; }
        //}
        loop {
            if exit == true { break; }
            let mut x = CONSOLE.lock();
            let byte = x.read_byte();

            if byte == b'\r' || byte == b'\n' {
                let mut input_mem: [&str; 64] = [""; 64];
                let command_in = Command::parse(
                    core::str::from_utf8(buffer.into_slice()).unwrap(),
                    &mut input_mem);
                kprint!("\n");

                match command_in {
                    Err(Error::TooManyArgs)=> {
                        kprintln!("error: too many arguments");
                    },
                    Err(Error::Empty) => {
                    },
                    Ok(command) => {
                        if command.path() == "echo" {
                            let num_args = command.args.len();

                            if num_args > 1 {
                                for string in command.args[1..num_args-1].iter() {
                                    kprint!("{} ", string);
                                }
                                kprintln!("{}", command.args[num_args-1]);
                            }
                        } else if command.path() == "pwd" {
                           pwd(&working_dir); 
                        } else if command.path() == "cd" {
                            cd(&command.args[1..], &mut working_dir);
                        } else if command.path() == "ls" {
                            ls(&command.args[1..], &working_dir);
                        } else if command.path() == "cat" {
                            cat(&command.args[1..], &working_dir);
                        } else if command.path() == "exit" {
                            exit = true;
                        } else {
                            kprintln!("unknown command: {}", command.path());
                        }
                    },
                }   
                break;
            } else {
                let mut x = CONSOLE.lock();
                
                if byte == 8 || byte == 127 {
                    if buffer.pop() == None {
                        x.write_byte(7);
                    } else {
                        x.write_byte(8);//(&[8, b' ', 8]); //not found
                        x.write_byte(b' ');
                        x.write_byte(8);
                    }
                } else if byte < b'\x20' || byte > 0x7E {
                    x.write_byte(7);
                } else {
                    if buffer.push(byte).is_err() {
                        //do nothing
                    } else {
                        x.write_byte(byte);
                    }
                }
            }
        }
    }

    fn cat(args: &[&str], working_dir: &PathBuf) {
        if args.len() != 1 {
            kprintln!("cat demands more");
        }

        let mut dir = working_dir.clone();

        dir.push(args[0]);

        let entry_return = FILESYSTEM.open(dir.as_path());
        match entry_return {
            Ok(x) => {
                if let Some(ref mut file) = x.into_file() {
                    loop {

                        let mut buffer = [0u8; 512];
                        use shim::io::Read;
                        match file.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(_) => {
                                match String::from_utf8(buffer.to_vec()) {
                                    Ok(k)=> kprint!("{}", k),
                                    Err(e) => kprintln!("{:?}", e),
                                }
                            }
                            Err(e) => kprint!("Failed to read file: {:?}", e)
                        }
                    }
                    kprintln!("");
                } else {
                    kprintln!("Not a file");
                }
            },
            Err(x) => { 
                kprintln!("path not found");
                return;
            }
        }
    }

    fn pwd(working_dir: &PathBuf) {
        kprintln!("{}", working_dir.as_path().display()); // is this a thing
    }

    fn cd(args: &[&str], working_dir: &mut PathBuf) {
        if args.len() == 0 {
            return;
        } else if args.len() != 1 {
            kprintln!("too many arguments");
            return;
        }

        if args[0] == "." {
        } else if args[0] == ".." {
            working_dir.pop();
        } else {
            let path = Path::new(args[0]);

            let mut new_dir = working_dir.clone();
            new_dir.push(path);
            let entry = FILESYSTEM.open(new_dir.as_path());
            match entry {
                Err(_) => {
                    kprintln!("cd: no such file or directory: {:?}", args[0]);
                    return;
                }, 
                Ok(x) => {
                    match x.as_dir() {
                        Some(y) => working_dir.push(path),
                        None => kprintln!("cd: no such file or directory: {:?}", args[0]),
                    }
                }
            }
        }
    }

    fn ls(args1: &[&str], working_dir: &PathBuf) {
        let mut args = args1.clone();
        let show_hidden = args.len() > 0 && args[0] == "-a"; //slick
        if show_hidden {
            args = &args[1..];
        }

        if args.len() > 1 {
            kprint!("ls: cannot access multiple directories at once, install BSD instead");
            return;
        }
        let mut dir = working_dir.clone();

        if !args.is_empty() {
            if args[0] == "." {
            } else if args[0] == ".." {
                dir.pop();
            } else {
                dir.push(args[0]);
            }
        }
        let entry = FILESYSTEM.open(dir.as_path());
        match entry {
            Err(_) => {
                kprintln!("ls: no such directory: {:?}", args[0]);
                return;
            }, 
            Ok(x) => {
                match x.into_dir() {
                    Some(y) => {
                        let mut entries = y.entries();
                        for things in entries.unwrap() {
                            if show_hidden || !things.metadata().hidden() {
                                print_entry(&things);
                            }
                        }
                    },
                    None => kprintln!("ls: no such directory: {:?}", args[0]),
                }
            }
        }
    }

    fn print_entry<F: Entry>(entry: &F) {
        /*fn write_bool(b: bool, c: char) {
            if b { kprint!("{}", c); } else { kprint!("-"); }
        }

        fn write_time<T: Time>(time: T) {
            kprint!("{:02}:{:02}:{:02} ", time.hour(), time.minute(), time.second());
        }
        fn write_date<D: Date>(date: D) {
            kprint!("{:02}/{:02}/{} ", data.month(), date.day(), date.year());
        }*/

        //write_bool(entry.is_dir(), 'd');
        //write_bool(entry.is_file(), 'f');
        //write_bool(entry.metadata().readonly(), 'r');
        //write_bool(entry.metadata().hidden(), 'h');
        //kprint!("\t");
        //write_date(entry.metadata().date_created());
        //write_time(entry.metadata().time_create());
        //write_date(entry.metadata().date_modified());
        //write_date(entry.metadata().date_accessed());
        //write_time(entry.metadata().time_accessed());
        //kprint!("\t");
        kprintln!("{}", entry.name());
    }
    kprintln!("exiting");
    return
}
