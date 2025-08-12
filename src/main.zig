const std = @import("std");
const Thread = std.Thread;
const Mutex = Thread.Mutex;

export fn zig_add(libp2pEvents: *Libp2pEvents, a: i32, b: i32, message: [*:0]const u8) i32 {
    const my = [_]u8{message[0]};
    // how to handle its alloc/dealloc or should new it?
    const newmessage = std.mem.span(message);
    libp2pEvents.push(newmessage) catch {};

    std.debug.print("zig add {d} {d} {s} {s} libp2pEvents={any}\n", .{ a, b, my, message, libp2pEvents });
    return a + b;
}

pub extern fn createNetwork(a: *Libp2pEvents, a: i32, b: i32) u32;
pub extern fn publishMsg(message: [*c]const u8, len: usize) void;

const Libp2pEvents = struct {
    mutex: Mutex,
    messages: std.ArrayList([]const u8),
    id: u32,
    const Self = @This();

    pub fn push(self: *Self, message: []const u8) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        try self.messages.append(message);
        std.debug.print("\npushed {s}, messages: {any}\n", .{ message, self.messages.items.len });
    }
};

fn startMainThreadNetwork(self_port: i32, connect_port: i32, libp2pEvents: *Libp2pEvents) !void {
    const p2p_ptr = createNetwork(libp2pEvents, self_port, connect_port);
    std.debug.print("rust_libp2p.dll: createNetwork({}, {}) = {}\n", .{ self_port, connect_port, p2p_ptr });

    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print(" startMainThreadNetwork sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
    }
}

pub fn main() !void {
    var args = std.process.args();
    _ = args.skip();

    const self_port = try std.fmt.parseInt(i32, args.next().?, 10);
    const connect_port = try std.fmt.parseInt(i32, args.next() orelse "-1", 10);
    var libp2pEvents = Libp2pEvents{ .mutex = Mutex{}, .messages = std.ArrayList([]const u8).init(std.heap.page_allocator), .id = 2345 };

    const thread = try Thread.spawn(.{}, startMainThreadNetwork, .{ self_port, connect_port, &libp2pEvents });
    _ = thread;

    // try startMainThreadNetwork(self_port, connect_port, &libp2pEvents);
    const allocator = std.heap.page_allocator;
    var i: i32 = 0;
    while (true) {
        i = i + 1;
        std.debug.print("main sleeping iteration: {d}\n", .{i});
        std.time.sleep(1000000000);
        if (connect_port > 0) {
            const message = try std.fmt.allocPrint(allocator, "peer sleeping iteration: {d}", .{i});
            publishMsg(message.ptr, message.len);
        }
    }
}
