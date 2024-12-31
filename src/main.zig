const std = @import("std");

export fn zig_add(a: i32, b: i32) i32 {
    std.debug.print("zig add\n", .{});
    return a + b;
}

pub extern fn startNetwork(a: i32, b: i32) i32;

pub fn main() !void {
    var args = std.process.args();
    _ = args.skip();

    const self_port = try std.fmt.parseInt(i32, args.next().?, 10);
    const connect_port = try std.fmt.parseInt(i32, args.next() orelse "-1", 10);

    std.debug.print("selfPort: {d} connectPort={d}", .{ self_port, connect_port });

    std.debug.print("rust_libp2p.dll: startNetwork({}, {}) = {}\n", .{
        self_port,
        connect_port,
        startNetwork(self_port, connect_port),
    });

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print("sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}
