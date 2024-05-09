---
title: "Writing an SDK in Zig, Part 1"
description: "Part 1 of writing an SDK for Axiom using the Zig programming language."
location: "Kiel, Germany"
published: "2024-05-09"
---

The first project I used Zig for was a rewrite of a custom static site generator
for the [Fire Chicken Webring](https://firechicken.club), you can read that post
here: [Thoughts on Zig](/blog/thoughts-on-zig).

Writing a small application is a lot easier than writing a library, especially
if you're hacking it together like I was.
So let's do something harder.

And because I work at [Axiom](https://axiom.co), we're going to write an SDK for
the [public API](https://axiom.co/docs/restapi/endpoints).
In this first part I'll set up the library and add a simpel `getDatasets` fn
to fetch all datasets the token has access to.

<!-- more -->

<em class="note">

We're using Zig 0.12. It might not work with a different version.

</em>

## Bootstrap the SDK

First, we create a directory called `axiom-zig` and run `zig init`:

```zsh
info: created build.zig
info: created build.zig.zon
info: created src/main.zig
info: created src/root.zig
info: see `zig build --help` for a menu of options
```

We also want to create a `.gitignore` to ignore the following folders:

```
/zig-cache
/zig-out
```

Perfect. Next step: Create a client struct in `root.zig`.
We'll need an Axiom API token to authenticate requests, a `std.http.Client` to
make requests and an `std.mem.Allocator` to allocate and free resources:

```zig
const std = @import("std");
const Allocator = std.mem.Allocator;
const http = std.http;
// We'll need these later:
const fmt = std.fmt;
const json = std.json;

/// SDK provides methods to interact with the Axiom API.
pub const SDK = struct {
    allocator: Allocator,
    api_token: []const u8,
    http_client: http.Client,

    /// Create a new SDK.
    fn new(allocator: Allocator, api_token: []const u8) SDK {
        return SDK{ .allocator = allocator, .api_token = api_token, .http_client = http.Client{ .allocator = allocator } };
    }

    /// Deinitialize the SDK.
    fn deinit(self: *SDK) void {
        self.http_client.deinit();
    }
}

test "SDK.init/deinit" {
    var sdk = SDK.new(std.testing.allocator, "token");
    defer sdk.deinit();
    try std.testing.expectEqual(sdk.api_token, "token");
}
```

Initially I had `deinit(self: SDK)` (without the pointer). Zig didn't like this
at all and led me down a rabbit hole of storing the `http.Client` as a pointer
too—once I found my way out and remembered I need a pointer everything worked.

I like that Zig encourages writing tests not only next to the source code (like
Go), not only in the same file (like Rust), but _next to the code it's testing_.

## Add getDatasets

### Create a model

Our first method will be `getDatasets`, which returns a list of Axiom datasets
([see api documentation](https://axiom.co/docs/restapi/endpoints/getDatasets)).

For that, we need a model:

```zig
pub const Dataset = struct {
    id: []const u8,
    name: []const u8,
    description: []const u8,
    who: []const u8,
    created: []const u8,
};
```

Don't worry about `created` being a datetime, we'll deal with that later.

### Add the `getDatasets` fn

Let's add a function to get the datasets to our `SDK` struct:

```zig
/// Get all datasets the token has access to.
/// Caller owns the memory.
fn getDatasets(self: *SDK) ![]Dataset {
    // TODO: Store base URL in global const or struct
    const url = comptime std.Uri.parse("https://api.axiom.co/v2/datasets") catch unreachable;

    // TODO: Draw the rest of the owl
}
```

We're taking a pointer to `SDK` called `self` again, this means that this is a
method you call on a created `SDK`. The `!` means it can return an error.
In a later post I want to go deeper into error handling, for now it can return
_any_ error.

Because there is no dynamic part of the URL, we can parse it at compile time
using `comptime`.
I like this explicit keyword, in Rust you need to rely on macros to achieve
something similar, or use
[lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs).

### Make the HTTP request

Let's open a connection to the server:

```zig
var server_header_buffer: [4096]u8 = undefined; // Is 4kb enough?
var request = try self.http_client.open(.GET, url, .{
    .server_header_buffer = &server_header_buffer,
});
defer request.deinit();
```

I wonder if 4kb is always enough for server headers. Especially in a library I
don't want it to fail because the server is suddenly sending more headers.

Axiom uses Bearer auth, so let's set the `Authorization` header:

```zig
var authorization_header_buf: [64]u8 = undefined;
// TODO: Store this on the SDK for better re-use.
const authorization_header = try fmt.bufPrint(&authorization_header_buf, "Bearer {s}", .{self.api_token});
request.headers.authorization = .{ .override = authorization_header };
```

An Axiom API is 41 characters, plus `Bearer `'s 7 characters equals 48 characters.
We're allocating 64 to be safe if it ever changes (it really shouldn't).

Also note that I'm calling `try fmt.BufPrint`; this will return the error
to the caller of our function (that's what the `!` indicating).

Finally, we can send the headers to the server and wait for a response:

```zig
try request.send();
try request.wait();
```

### Parse the JSON-response

First, we need to read the body into a buffer:

```zig
var body: [1024 * 1024]u8 = undefined; // 1mb should be enough?
const content_length = try request.reader().readAll(&body);
```

Same issue as with the server headers: What is a good size for a fixed buffer
here?

I've tried doing this dynamically with
`request.reader().readAllAlloc(...)`, but parsing the JSON with this allocated
memory relied on the allocated `[]const u8` for string values.
This means as soon as I deallocated the body, all string values in the returned
JSON were invalid (use-after-free). Yikes.

So let's call `json.parseFromSlice` with our body:

```zig
const parsed_datasets = try json.parseFromSlice([]Dataset, self.allocator, body[0..content_length], .{});
defer parsed_datasets.deinit();
```

Now we need to copy the memory out of the `parsed_datasets.value` to prevent it
from being freed on the `parsed_datasets.deinit()` above and return it:

```zig
const datasets = try self.allocator.dupe(Dataset, parsed_datasets.value);
return datasets;
```

### Write a test

And finally we'll write a test where we initialize the SDK, get the datasets
and ensure `_traces` is the first one returned.
Once I set up CI, I'll create an Axiom org just for testing so we can be sure
which datasets are returned.

```
test "getDatasets" {
    const allocator = std.testing.allocator;

    const api_token = try std.process.getEnvVarOwned(allocator, "AXIOM_TOKEN");
    defer allocator.free(api_token);

    var sdk = SDK.new(allocator, api_token);
    defer sdk.deinit();

    const datasets = try sdk.getDatasets();
    defer allocator.free(datasets);

    try std.testing.expect(datasets.len > 0);
    try std.testing.expectEqualStrings("_traces", datasets[0].name);
}
```

## Next steps

In the next part I'll add `createDataset`, `updateDataset` and `deleteDataset`,
initial error handling and show how you can import the library in a Zig project.
