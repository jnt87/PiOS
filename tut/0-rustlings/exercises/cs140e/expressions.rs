// FIXME: Make me pass! Diff budget: 10 lines.
// Do not `use` any items.

// Do not change the following two lines.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
struct IntWrapper(isize);
impl Into<usize> for IntWrapper {
    fn into(self) -> usize {
        self.0 as usize
    }
}
// Implement a generic function here
// fn max...
// my code
fn max<N>(arg1: N, arg2: N) -> usize where N: Into<usize> + Copy {
    if arg1.into() < arg2.into() { arg2.into() } else { arg1.into() }
}

#[test]
fn expressions() {
    assert_eq!(max(1usize, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(IntWrapper(120), IntWrapper(248)), IntWrapper(248).into());
}
