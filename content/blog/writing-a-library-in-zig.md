---
title: "Writing a Library in Zig"
description: "Part 1 of writing an SDK for Axiom usnig the Zig programming language."
location: "Kiel, Germany"
published: "2024-05-03"
---

The first project I used Zig for was a rewrite of a custom static site generator
for the [Fire Chicken Webring](https://firechicken.club), you can read that post
here: [Thoughts on Zig](/blog/thoughts-on-zig).

Writing a small application is a lot easier than writing a library, especially
if you're hacking it together like I was.
So let's do something harder.

And because I work at [Axiom](https://axiom.co), we're going to write an SDK for
the [public API](https://axiom.co/docs/restapi/endpoints).

<!-- more -->

We're using Zig 0.12, if you use something else it might not work.

## Bootstrapping

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

Perfect. Next step: Create a client struct in `root.zig`:

```zig
const std = @import("std");
const Allocator = std.mem.Allocator;
const http = std.http;
// We'll need these later:
const fmt = std.fmt;
const json = std.json;

/// SDK provides methods to interact with the Axiom API.
pub const SDK = struct {
    api_token: []const u8,
    http_client: http.Client,

    /// Create a new SDK.
    fn init(allocator: Allocator, api_token: []const u8) SDK {
        return SDK{ .api_token = api_token, .http_client = http.Client{ .allocator = allocator } };
    }

    /// Deinitialize the SDK.
    fn deinit(self: *SDK) void {
        self.http_client.deinit();
    }
};

test "Create SDK" {
    var sdk = SDK.new(std.testing.allocator, "token");
    defer sdk.deinit();
    try std.testing.expectEqual(sdk.api_token, "token");
}
```

The struct takes an allocator and an API token and stores both for future use.

## Prepare our first method

Let's start with something simple: Getting the
[list of datasets](https://axiom.co/docs/restapi/endpoints/getDatasets).

We need a model. Don't worry about `created` being a datetime, we'll deal with
that later.

```zig
pub const Dataset = struct {
    id: []const u8,
    name: []const u8,
    description: []const u8,
    who: []const u8,
    created: []const u8,
};
```

We also define the API URL at the top of the file:

```zig
const axiom_api_url = std.Uri.parse("https://axiom.co") catch unreachable;
```

## The first method

Orhun Parmaksız has a great guide on making HTTP requests in
[part 4 of their Zig Bits series](https://blog.orhun.dev/zig-bits-04/),
highly recommend checking that out.

Let's add a function to get the datasets to our `SDK` struct:

```zig
/// Get all datasets the token has access to.
fn get_datasets(self: *SDK) ![]Dataset {
    const url = comptime axiom_api_url.resolve_inplace("/api/v1/datasets") catch unreachable;

    // TODO: Draw the rest of the owl
}
```

We're taking a pointer to `SDK` called `self` again, this means that this is a
method you call on a created `SDK`. The `!` means it can return an error (we'll
get to that later).

Because there is no dynamic part of the URL, we can parse it at compile time
using `comptime`.

Axiom uses Bearer auth, so we need an `Authorization` header:

```zig
var headers = http.Headers{ .allocator = self.allocator };
defer headers.deinit();

var authorization_header_buf: [64]u8 = undefined;
defer self.allocator.free(authorization_header_buf);
const authorization_header = try fmt.bufPrint(&authorization_header_buf, "Bearer {s}", .{self.token});
try headers.append("Authorization", authorization_header);
```

An Axiom API is 41 characters, plus `Bearer `'s 7 characters equals 48 characters.
We're allocating 64 to be safe if it ever changes (it really shouldn't).
Because we need the same token header for every request, it should really be
allocated on `init` and stored on the client. We'll do that later.

Also note that I'm calling `try headers.append`; this will return the error
to the caller of our function (that's what the `!` is for).

Finally, we can send the request to the server:

```zig
var request = try self.client.request(.GET, url, headers, .{});
defer request.deinit();

try request.start();
try request.wait();
```

Phew, that was work. Next, let's check the status code and parse the JSON:

```
const body = try request.reader().readAllAlloc(self.allocator, 4096);
defer self.allocator.free(body);
```

If the returned body is over 4MB, this returns an error.
