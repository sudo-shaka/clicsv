# CLICSV

I was tired of having to open up a spreadsheet editor when working with data in the terminal and wanted to get more familiar with the rust programming language so I decided to make this. It's a command-line csv text editor written in rust. Currently a work in progress and needs refactoring, but is functional. 

![Screenshot](https://user-images.githubusercontent.com/68864205/128723885-d5906592-96b1-462c-89b2-635ed71cb03c.png)

# Installation
From souce: <br /> <br />
with rust installed run: <br />
cargo install clicsv<br />
<br />

# Usage
Enter/Return = Put cell into edit mode <br />
Control+Q = quit <br />
Control+C = copy highlighted cells <br />
Control+P = paste selection <br />
Control+S = save file <br />
Arrow Keys (Direction) = scroll through cells <br />
Control+Direction = sigular highlight <br />
Shift+Direction = highlight from cell to terminus of that direction <br />

# Things to add
Undo functionality <br />
Multiple cell delections <br />
Fix scrolling going from left to right.