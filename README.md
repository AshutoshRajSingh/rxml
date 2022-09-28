# RXML
RXML (Pronounced rexml) is a highly makeshift, and very unstable XML parser written in rust by a cs college senior who didn't have anything better to do.

## Installation
1. Clone the repository using `git clone https://github.com/AshutoshRajSingh/rxml.git`
2. In your `Cargo.toml` file, make modifications under the `[dependencies]` section:  
    ```
    [dependencies]
    rxml = { path = "<path to root of cloned repo without angular brackets>" }
    ```
## Example
`src/main.rs`
```rust
use rxml::RXML;
fn main() {
    let some_xml = String::from("<xml version='1.0' encoding='utf-8'> <person> <name> John </name> <age> 25 </age> <ssn> 5 </ssn> </person> </xml>");

    let rxml = RXML::new(some_xml);
    let top_node = rxml.parse().unwrap();

    println!("{:#}", top_node);
}
```
Running above code will output: 
```
<xml {"version": "1.0", "encoding": "utf-8"}>
 <person {}>
  <ssn {}> '5'
  <age {}> '25'
  <name {}> 'John'
```

## Features:
- Simple API (only one function to call)
- Maybe faster because it doesn't support any of that xml schema stuff, everything is a string.
- Gives you a tree hierarchy of your xml data, traverse it yourself.
- Does not cry like a baby on seeing something like `<person name='John' oopsie these attributes have no values>`, just ignores valueless attribs and moves on with its life.
- You can even throw in something like `<person> John <age> 45 </age></person>` and it'll just associate `John` with the tag `<person>`

## FAQs

Q. Is it fast?  
A. Definitely faster than ~~your..~~ trying to figure out how to parse xml using regex and then failing.

Q. Will it crash?  
A. I think there's one scenario where it might but I won't tell you.

Q. Is it reliable?  
A. No.

Q. Is it usable?  
A. If your use case is a meme.

Q. Where is the documentation?  
A. Lol.

Q. Can I use it in a production environment?  
A. RXML should not be used in any environment.