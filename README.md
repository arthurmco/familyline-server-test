# familyline-server-test

A little test server used to implement protocol ideas for
[Familyline](https://github.com/arthurmco/familyline), a game I am
developing.


Also a way for me to learn Rust.

This code is not too good, but I will probably make it clearer.

Also, no test suite yet. My [other Rust experimental
program](https://gist.github.com/arthurmco/92ab10eb55cbc332132010bb5b46502a)
has some.

## Actual state

Currently, I have only:

 - the basic discovery protocol, a very striped-down version of SSDP,
   that the client will use to find the server. The server binds to a
   multicast address, the same as SSDP, but to a different port to avoid
   issues.
 
 - most of the HTTP protocol that the game will use to query things to
   the server when on lobby, and probably to propose suggestions, like
   map type and game type. Also you will be able to download maps,
   like Quake game servers let you.  
   There are some properties of the game, like game mode for example,
   that I will implement after I have a more or less working client on
   the game itself
   

The in-game protocol, that the game will use to send his inputs and
receive other players' input, send and receive chat data,
synchronization packets and verification packets, is not
implemented. Not a single bit *yet*.  
But I plan to use something like Protocol Buffers or FlatBuffers to
encode it. They need you to define  a schema, and they encode the data
in a more or less common format. The schema definition will of course
help development.

-------

And that's it. Suggestions and criticism are very welcome, but please
do not be an �sshole. I am learning.

