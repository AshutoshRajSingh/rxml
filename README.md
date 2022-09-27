# RXML
RXML (Pronounced rexml) is a highly makeshift, and very unstable XML parser written in rust by a cs college senior who didn't have anything better to do.

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