# CLICSV

I was tired of having to open up a spreadsheet editor when working with data in the terminal and wanted to get more familiar with the rust programming language so I decided to make this. It's a command-line csv text editor written in rust. Currently a work in progress and needs refactoring, but is functional. 

![Screenshot](https://user-images.githubusercontent.com/68864205/128723885-d5906592-96b1-462c-89b2-635ed71cb03c.png)

# Installation
From souce: (this will be the most up to date)<br /> <br />
with cargo installed, run: 
```
cargo install clicsv
```
<br />

If you are on NetBSD, a package is available from the official repositories.
To install it, simply run:
```
pkgin install clicsv
```

# Usage
Enter/Return = Put cell into edit mode <br />
Control+Q = quit <br />
Control+C = copy highlighted cells <br />
Control+X = cut highlighted cells <br />
Control+P = paste selection <br />
Control+S = save file <br />
Control+Z = undo <br />
Arrow Keys (Direction) = scroll through cells <br />
Control+Direction = singular highlight <br />
Shift+Direction = highlight from cell to terminus of that direction <br />
