# Familyline network protocol

This is the network protocol for the second part of the connection.
After you type `connect`, the server will open a listening socket and
wait for the client connection at a certain port.

You will need to send messages in the following structure:

## Packet Structure

It goes like this:

| Field        | Size | Description                               |
| ------------ | ---- | ----------------------------------------- |
| magic        | 4    | The magic header for the message          |
| flags        | 4    | Some flags for the message. Currently 0   |
| checksum     | 4    | The CRC32 checksum of the whole message   |
| payloadsize  | 4    | Size of the payload, in bytes             |
| payload      | n    | The payload                               |

The magic header is 'FAMI' in ascii, little-endian

The payload is a FlatBuffers buffer, and is more or less like this:

```flatbuffers

table Packet {
   tick: ulong;
   source_client: ulong;
   dest_client: ulong;
   timestamp: ulong;
   message_id: ulong;

   message: Message;
};

```

 - The tick is the tick that the message was sent. If sent before the
   game start, the tick will always 0.

 - timestamp is the unix timestamp of the time the message was sent, in
   UTC.

 - source\_client and dest\_client are where the message come from and
   where it will go. If one of those fields are 0, it means that the
   source, or destination, is the server.

 - the message_id is used to track a pair of `Request` and `Response`


 - message is the content of the message.


## Message structure

We have some important message types

```flatbuffers

union Message {
    StartRequest,
    StartResponse,


    ErrorRequest,
    ErrorResponse,
}

```

### StartRequest & StartResponse

They are sent first.

The client sends them when it connects to the address passed in from
the `/connect` endpoint.

The message is:

```flatbuffers

table StartRequest {
    clientid: ulong,
    token: string
}

```

The client will send the response after. It will be one for each
client connected after that, then one final when all clients are
connected.

```flatbuffers

table StartResponse {
    client_ack: ulong,
    all_clients_ack: bool,
}


```

`client_ack` is the ID of the client acknowledged. If you sent the
`StartRequest` right before, it will probably be you. (The first
StartResponse will always be for you). If not, than it will be not,
and the message that is not only serves for the client to know that
other clients connected.

`all_clients_ack` flags that all clients have been connected.


### LoadingRequest & LoadingResponse

They are sent after the StartResponse acknowledges all clients.

They are equal

```flatbuffers

table LoadingRequest {
    loading_percent: ushort
}

table LoadingResponse {
    loading_percent: ushort
}

```

They represent the percentage of the loading, from 0 to 100. In
practice, you will only send the 0 when you start and the 100 when you finish

### GameStartRequest & GameStartResponse

It is sent right after you got the LoadingResponse with the 100%
loading percent.

The response will come only when all clients send the
GameStartRequest.

```flatbuffers

table GameStartRequest {
    reserved: uint
}

table GameStartResponse {
    reserved: uint
}

```

The reserved value is always zero.


### SendInputRequest & SendInputResponse

The SendInputRequest is the only request the server can send.

Usually, the client send the SendInputRequest, then the server sends
the SendInputResponse, then the server redistributes this request to
all clients, and all clients reply with the SendInputResponse.

```flatbuffers

table SendInputRequest {
    client_from: ulong
    input: InputType
}

table SendInputResponse {
    client_from: ulong,
    ack: bool
}

```
 
  - `client_from` is the client ID of where the packet came
  - `input` is the input; the InputType field is from [here](https://github.com/arthurmco/familyline/blob/master/src/common/input_serialize.fbs)




### ErrorRequest & ErrorResponse

They serve to send errors.

An ErrorRequest is sent by the client to the server, and an
ErrorResponse is sent by the server to all the clients.

```flatbuffers

table ErrorRequest {
    error: Error,
    entity_id: ulong,
    origin_message_timestamp: ulong    
}

table ErrorResponse {
    client: ulong,

    error: Error,
    entity_id: ulong,
    origin_message_id: ulong    
}

```

 - `client` is the client that sent the error, or 0 if the server sent
   the error.
 - `error` is the error value
 - `entity_id`, if non-zero, is the entity that caused the error.
 - `origin_message_timestamp`, if non-zero, is the ID of the
   message that caused the error.


### DisconnectRequest and DisconnectResponse

They are send whenever a client disconnects.

The disconnecting entity sends the disconnect request, then everyone
receives the disconnect response, including the one that disconnected.

```flatbuffers

table DisconnectRequest {
    player_id: ulong,
}

table DisconnectResponse {
    player_id: ulong,
}

```
