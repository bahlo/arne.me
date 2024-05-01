---
title: "Thoughts on Zig"
published: "2024-05-01"
location: "Frankfurt, Germany"
description: "I've started writing Zig. These are my thoughts."
---

Zig is a programming language designed by Andrew Kelley.
The [official website](https://ziglang.org) lists three principles of the
language:

- No hidden control flow.
- No hidden memory allocations.
- No preprocessor, no macros.

For someone like me coming mostly from Rust, Go and TypeScript, this is different—and different is interesting, so I wanted to know what it feels like
to write code in it.

Here are my thoughts after 3 nights of using Zig to rewrite the static site generator[^1] I use for the [Fire Chicken Webring](https://firechicken.club).
Note that this is a limited use case and only scratches the surface of the
language.

<!-- more -->

## Explicit allocation

This is one of the biggest differentiators of Zig.
Go doesn't force you to think at all, Rust forces you to think about ownership
and Zig forces you to think about allocations.
If you want to allocate memory, you need to pass an allocator to it and
remember to free it afterwards.

One thing that I stumbled upon a lot:
Sometimes there is a `.deinit()` function on the returned struct, sometimes
that method takes an `allocator`, sometimes you need to `allocator.free(value)`
and sometimes it returns an enum or a struct, and you need to figure out which
values you have to free.

## Documentation

If you write Zig, you'll find yourself reading Zig a lot to understand how to
use a function, which resources you must free, and possibly, why that function
panics.
There is no generated documentation like [docs.rs](https://docs.rs) or [pkg.go.dev](https://pkg.go.dev); if you want to know which methods a library has, look at the source.

Here are some resources other than the
[source code](https://github.com/ziglang/zig/tree/master)
that I found useful to get started:

- [In-depth Overview](https://ziglang.org/learn/overview/)
- [Zig Language Reference](https://ziglang.org/documentation/)
- [Zig by Example](https://zig-by-example.com)
- [Zig Cookbook](https://cookbook.ziglang.cc)
- [Zig Guide](https://zig.guide)
- [Zig std documentation](https://ziglang.org/documentation/master/std/#std)

Another reason you might need to look at the source code of function you're
calling is confusing error messages.

## Errors

The error messages of the Zig compiler can be very hard to figure out.
Here's an example:

```sh
$ zig build run
Segmentation fault at address 0x102ee6000
Panicked during a panic. Aborting.
```

Generally, if you're used to Rust's _exceptional_ error handling, this is rough.

Once I got an error from the standard library and only noticed after
reading the source code that `ArrayList` is not a supported type to pass to the
given function.
Another time, the templating library I've temporarily used randomly panicked
with an out-of-bounds after doing a nested loop.

## Libraries

There are a bunch of libraries for Zig (see [awesome-zig](https://github.com/zigcc/awesome-zig)) and I can only talk about
the one's I've tried, but most of the libraries I've looked at are either
archived, a thin wrapper around a C library, heavy WIP and barely usable, or have weird error scenarios.

This lead to me implementing my own
[shitty datetime function](https://github.com/bahlo/firechicken.club/blob/78491b3c2b04d04c4f0bfdce3b360c8081837683/src/main.zig#L241-L320)[^2]
and using `std.fmt` instead of templating.

I believe these are due to the immaturity of the language and ecosystem, but I
wouldn't be surprised if people started building their own libraries, which they
take everywhere.

## Strings

We have to talk about strings. Zig has none; if you want a string, use
`[]const u8`. You also can't compare that type with `==`, you need to use a
specific function[^3].

Initially I found this irritating—why not introduce a string type that is
`[]const u8` under the hood and overload the `==` operator?
I think it would increase developer experience, but does it fit into Zig?

## No magic

Remember the three idioms from the beginning of the article?
Zig is huge on being _transparent_, i.e., no magic; the code you read is what
happens.

And I appreciate that.
In Rust, it's common to build abstractions to hide boilerplate logic
(e.g. using macros to generate the deserialization logic of a struct), in Go
it's common to generate code to do this.
Zig doesn't have any of that (though I guess you could generate Zig code).
I'm not sure how well that scales in big codebases, but I think it's
interesting.

## Conclusion

I like Zig. For a bigger project or something that needs async[^4], I'll still
reach for Rust for its safety features and vibrant ecosystem, but for small
projects, it's fun to reach for an interesting language.

[^1]: See [Why You Should Write Your Own Static Site Generator](https://arne.me/blog/write-your-own-ssg)
[^2]: Maybe don't look too closely.
[^3]: `std.mem.eql`
[^4]: Async functions have been removed from Zig at this time.
