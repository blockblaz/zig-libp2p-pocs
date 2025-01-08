const std = @import("std");

export fn zig_add(a: i32, b: i32) i32 {
    std.debug.print("zig add\n", .{});
    return a + b;
}

pub extern fn createNetwork() u32;
pub extern fn startNetwork(a: u32, a: i32, b: i32) void;

pub fn main() !void {
    var args = std.process.args();
    _ = args.skip();

    const self_port = try std.fmt.parseInt(i32, args.next().?, 10);
    const connect_port = try std.fmt.parseInt(i32, args.next() orelse "-1", 10);

    const p2p_ptr = createNetwork();
    std.debug.print("starting selfPort: {d} connectPort={d}", .{ self_port, connect_port });
    startNetwork(p2p_ptr, self_port, connect_port);

    std.debug.print("rust_libp2p.dll: createNetwork({}, {}) = {}\n", .{ self_port, connect_port, p2p_ptr });

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print("sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}
