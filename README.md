# Json-parser in Rust 🦀

A simple json/jsonc parser in rust,

The parsing is done in 2 pass, In first pass Lexing is done to get the Vector of Tokens and in second the actual parsing. For parsing i just implemented a simple recursive descent parser.

The grammar i used can be found [here](./json-grammar.md)

TODO: Implement de-serialization

## Showcase

![image](https://github.com/user-attachments/assets/0851d38c-b895-4120-b7e3-3c4175fa5699)

Blazingly fast 🚀

Tested on a 700k line JSON [file](./test-json/huge.json)

![image](https://github.com/user-attachments/assets/268d49a2-9318-4a37-a824-2b5e001111ed)

## Things i should have done differently

1. [DONE] Better error handling, in tokens i should have captured there position in the file, so at the time of errors during
   tokenisation i could have reported them.
2. At the time of tokenization using an iterator became a pain in the ass, just a vector of chars would have made life
   easier i think but idk.
3. [DONE Partially] Better error handling at the time of parsing.

## References

- [json.org](https://www.json.org/json-en.html) For the json spec and grammar.
- [Creating a compiler](https://www.youtube.com/playlist?list=PLUDlas_Zy_qC7c5tCgTMYq2idyyT241qs) Randomly stumbled upon this on youtube gave me nice ideas on how to get started on lexing
- [Writing a Simple Parser in Rust](https://adriann.github.io/rust_parser.html) Referred to this to get started on the recursive descent parser
