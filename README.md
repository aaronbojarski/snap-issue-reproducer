# snap-issue-reproducer
A minimal Rust application that reproduces an issue when using the SNAP service.

Contains both a server and a client that communicate over SCION using the `scion-sdk`. The client uses the SNAP underlay. The server uses UDP underlay.
The client sends a ping message every 100ms to the server, which reflects the same message back to the client.

## Observed Issue
After some time (usually ~30 seconds), the client stops receiving responses from the server. The server continues to receive messages from the client, but the client does not receive any responses. Tcpdump on the server side shows that the server is sending responses back to the client, but they never arrive at the client. (At least no QUIC packet with the appropriate size is observed.) I therefore assume that the SNAP service is dropping the packets for some reason. The QUIC connection to the SNAP stays alive the entire time.

## Building and Running
Requires Rust and Cargo.
```bash
cargo build
```

To run the server:
```bash
./snap-issue-reproducer server --listen SERVER_ADDRESS --endhost-api http://endhost-api.example.com:5001
```

To run the client:
```bash
./snap-issue-reproducer client SERVER_ADDRESS --endhost-api http://endhost-api.example.com:5001 --snap-token "YOUR_SNAP_TOKEN"
```