# Jakebox

Jakebox is meant to be a game where you try to make your friends laugh by answering questions in the funniest way possible. Right now, JakeBox is in active development so many features are not available yet.

## Planned Features

- [X]  CLI
- [ ]  Web GUI
- [ ]  Lua scripting
- [ ]  Official hosting

## How to use
### Server hosting
Local hosting for testing the client on the same machine:
``` bash
jakebox server --url 127.0.0.1:7777
```
### Client CLI
Hosting the game (local example):
``` bash
jakebox client host --url http://127.0.0.1:7777 --username "John Doe"
```
This should output `Connection Successful. Join code is XXXX`, where XXXX is the code that you need to use to join that game.
To join a game, run:
``` bash
jakebox client join --url http://127.0.0.1:7777 --username "John Doe" --join-code XXXX
```
Right now, this will do nothing. This is because no event is active. To activate an event, go to another terminal and run: 
``` bash
jakebox client host --url http://127.0.0.1:7777 --username "John Doe" start-prompt --join-code XXXX --prompt "Some question"
```
Now if you go back to the other terminal, you will see the question pop up. You can now answer it.
