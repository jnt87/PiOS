// FIXME: Make me compile. Diff budget: 12 line additions and 2 characters.

#[derive(Debug)]
struct ErrorA;
#[derive(Debug)]
struct ErrorB;

#[derive(Debug)]
enum Error {
    A(ErrorA),
    B(ErrorB)
}

fn do_a() -> Result<u16, ErrorA> {
    let x: u16 = 3;
    Ok(x)
}

fn do_b() -> Result<u32, ErrorB> {
    let x: u32 = 5;
    Ok(x)
}

fn do_both() -> Result<(u16, u32), Error> {
    Ok((do_a().unwrap(), do_b().unwrap()))
}

fn main() { }
