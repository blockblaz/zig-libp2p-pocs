const std = @import("std");
const Thread = std.Thread;
const Mutex = Thread.Mutex;
const Allocator = std.mem.Allocator;

export fn zig_add(libp2pEvents: *Libp2pEvents, a: i32, b: i32, message_ptr: [*]const u8, message_len: usize) i32 {
    // how to handle its alloc/dealloc or should new it?
    const message: []const u8 = message_ptr[0..message_len];
    libp2pEvents.push(message) catch {};
    // _ = libp2pEvents;

    std.debug.print("zig add {d} {d} {s} {d}\n", .{ a, b, message, message_len });
    return a + b;
}

pub extern fn createNetwork(a: *Libp2pEvents, a: i32, b: i32) u32;
pub extern fn publishMsg(message: [*c]const u8, len: usize) void;

const Libp2pEvents = struct {
    allocator: Allocator,
    mutex: Mutex,
    messages: std.ArrayList([]const u8),
    id: u32,
    const Self = @This();

    pub fn push(self: *Self, message: []const u8) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        const zig_message = try self.allocator.alloc(u8, message.len);
        std.mem.copyForwards(u8, zig_message, message);
        try self.messages.append(zig_message);
        std.debug.print("\npushed {s}, messages: {any}\n", .{ zig_message, self.messages.items.len });
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
    const allocator = std.heap.page_allocator;

    var libp2pEvents = Libp2pEvents{ .allocator = allocator, .mutex = Mutex{}, .messages = std.ArrayList([]const u8).init(std.heap.page_allocator), .id = 2345 };

    const thread = try Thread.spawn(.{}, startMainThreadNetwork, .{ self_port, connect_port, &libp2pEvents });
    _ = thread;

    // try startMainThreadNetwork(self_port, connect_port, &libp2pEvents);
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
