use stack_vec::StackVec;

use crate::console::{kprint, kprintln, CONSOLE};
use shim::io;

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
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    loop {
        kprint!("{}", prefix);
        let mut buf_mem = [0u8; 512];
        let mut buffer = StackVec::new(&mut buf_mem);

        loop {

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
                        kprintln!("needs less than 64 args");
                    },
                    Err(Error::Empty) => {
                    },
                    Ok(command) => {
                        if command.path() == "echo" {
                            let num_args = command.args.len();

                            if num_args > 0 {
                                for string in command.args[..num_args-1].iter() {
                                    kprintln!("{} ", string);
                                }
                                kprintln!("{}", command.args[num_args-1]);
                            }
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
                        //do nothing
                    } else {
                        x.read_byte();
                        x.write_byte(8);//(&[8, b' ', 8]); //not found
                        x.write_byte(b' ');
                        x.write_byte(8);
                    }
                } else if byte < 32 || byte == 255 {
                    //do nothing
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
}
