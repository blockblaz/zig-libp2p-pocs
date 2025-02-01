const std = @import("std");

export fn zig_add(a: i32, b: i32, message: [*:0]const u8) i32 {
    const my = [_]u8{message[0]};
    std.debug.print("zig add {d} {d} {s} {s}\n", .{ a, b, my, message });
    return a + b;
}

pub extern fn createNetwork() u32;
pub extern fn startNetwork(a: u32, a: i32, b: i32) void;
pub extern fn createAndStartNetwork(a: i32, b: i32) void;

fn startMainThreadNetwork(self_port: i32, connect_port: i32) !void {
    const p2p_ptr = createNetwork();
    std.debug.print("starting selfPort: {d} connectPort={d}\n", .{ self_port, connect_port });
    startNetwork(p2p_ptr, self_port, connect_port);
    std.debug.print("rust_libp2p.dll: createNetwork({}, {}) = {}\n", .{ self_port, connect_port, p2p_ptr });

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print(" startMainThreadNetwork sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}

fn startThreadedNetwork(self_port: i32, connect_port: i32) !void {
    std.debug.print("starting selfPort: {d} connectPort={d}\n", .{ self_port, connect_port });
    createAndStartNetwork(self_port, connect_port);
    std.debug.print("rust_libp2p.dll: createAndStartNetwork({}, {})\n", .{ self_port, connect_port });

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print(" startThreadedNetwork sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}

pub fn main() !void {
    var args = std.process.args();
    _ = args.skip();

    const self_port = try std.fmt.parseInt(i32, args.next().?, 10);
    const connect_port = try std.fmt.parseInt(i32, args.next() orelse "-1", 10);

    const thread = try std.Thread.spawn(.{}, startThreadedNetwork, .{ self_port, connect_port });
    _ = thread;

    // try startMainThreadNetwork(self_port, connect_port);

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print("main sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}
