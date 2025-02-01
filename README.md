# zig-libp2p-pocs

1. build: zig build --summary all
2. start peer 1 (just listens on 9001): ./zig-out/bin/zig-exe 9001 -1
3. start peer2 (listens on 9002 and connect to peer 1 on 9001): ./zig-out/bin/zig-exe 9002 9001