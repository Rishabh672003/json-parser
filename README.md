# Json-parser

A simple json/jsonc parser in rust,
The parsing is done in 2 pass, In first pass Lexing is done to get the Vector of Tokens and in second the actual
parsing. For parsing i just implemented a simple recursive descent parser.
The grammar i used can be found [here](./json-grammar.md)

## Showcase

![image](https://github.com/user-attachments/assets/0851d38c-b895-4120-b7e3-3c4175fa5699)

Tested on a 700k line JSON [file](./test-json/huge.json)

![image](https://github.com/user-attachments/assets/268d49a2-9318-4a37-a824-2b5e001111ed)

