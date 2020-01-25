// FIXME: Make me pass! Diff budget: 30 lines.

#[derive(Default)]
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    //my code
    fn string<S>(&mut self, a: S) -> &mut Self where S: Into<String> {
        self.string = Some(a.into());
        self
    } 
    //my code
    fn number(&mut self, b: usize) -> &mut Self {
        self.number = Some(b);
        self
    }
}

impl ToString for Builder {
    // Implement the trait
    fn to_string(&self) -> String {
        let mut new_str = String::from("");
        let y = self.string.as_ref();
        if y != None {
            new_str.push_str(self.string.as_ref().unwrap());
        }
        let x = self.number.as_ref();
        if x != None && y != None {
            new_str.push_str(" ");
        }
        if x != None {
            new_str.push_str(&self.number.as_ref().unwrap().to_string());
        }
        new_str as String
    }
}

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");
    
    let just_str = Builder::default().string("hi");
    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
