// FIXME: Make me pass! Diff budget: 25 lines.

#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u64),
    Minutes(u64),
}

impl PartialEq for Duration {
    fn eq(&self, other: &Duration) -> bool {
        let mut y = 0;
        let mut z = 0;
        match &self {
            Duration::MilliSeconds(r) => y = *r,
            Duration::Seconds(r) => y = *r * 1000,
            Duration::Minutes(r) => y = *r * 1000 * 60,
        }
        match other {
            Duration::MilliSeconds(r) => z = *r,
            Duration::Seconds(r) => z = *r * 1000,
            Duration::Minutes(r) => z = *r * 1000 * 60,
        }
        if y == z {
            true
        } else {
            false
        }
    }
}

#[test]
fn traits() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}
