---
title: "Why You Should Write Your Own Static Site Generator"
description: "I rewrote my personal website using basic libraries and the flexibility is incredible."
published: "2023-11-03"
location: "Frankfurt, Germany"
---

I've used [a](https://jekyllrb.com) [lot](https://gohugo.io) 
[of](https://www.11ty.dev) [static](https://www.getzola.org) 
[site](https://nextjs.org) [generators](https://astro.build) in the past and 
they all have their own features and quirks, but most importantly you have to
architect your website to match what the framework expects.

Since yesterday this website is powered by my own SSG[^1].
It's not meant to be reusable, it's just normal code—parsing and generating 
files for this specific website.

And oh boy do I love it.

<!-- more -->

## Why??

When Vercel released [Next.js 14](https://nextjs.org/blog/next-14) recently, 
some friends I've talked to where still on Next.js 12 and really felt the 
pressure to upgrade to not fall behind even more.
This made me think about the longevity and robustness of my website and so I 
decided I don't want to depend on other people's decisions and run after version
upgrades I don't care about.

And even if your content is Markdown and media, almost everything around it
needs to be updated when switching frameworks—sometimes even when upgrading.
When I used Astro before, I wanted to statically generate OG images and, after 
some research, managed to build it and even wrote
[an article](/articles/static-og-images-in-astro) explaining how. 
You loose all of this custom logic.

Plus, you get to choose your own stack. 
Want to write your content in AsciiDoc? No-one can stop you!

I know what you're thinking and you're right, it's _way_ more work than using 
something that already exists.
But it's also so much more fun.
You can do anything with this, and you don't need to read documentation or try
to understand other people's architectural decisions—just start writing code!

## Okay, tell me how you did it

After contemplating to build something dynamic[^2] for search without 
JavaScript, I decided to stay with a static site. 
It's faster and you don't have to worry about security or stability.
And of course I choose the best programming language on the planet, Rust (my 
beloved). 
Wait, come back, this is not a Rust post!

A static site generator mostly needs to do five things:

1. Convert markdown to HTML
1. Render HTML templates
1. Compile CSS
1. Generate RSS feeds
1. Generate a sitemap

And of these you might not even need the last three.
Surely your favourite programming language has a Markdown parser and a 
templating engine.

For Rust I chose these crates[^3]:

- [comrak](https://crates.io/crates/comrak) and [gray_matter](https://crates.io/crates/gray_matter) for parsing Markdown and the frontmatter
- [maud](https://maud.lambda.xyz) for compile-time templates
- [grass](https://crates.io/crates/grass) for SCSS compilation
- [rss](https://crates.io/crates/rss) for (you guessed it) generating RSS feeds
- [quick-xml](https://crates.io/crates/quick-xml) for generating the sitemap

Once I got the tools I needed, I just started writing software.
I built something to parse my content, something to render all the different 
templates I needed, something to generate the RSS, something to generate a 
sitemap and…that's it, really!

And when it's time to add OG images to this website, I can choose the best
libraries and just build it.

## What now?

I hope this article left you either validated that rolling your own not worth it 
and you want to keep using a framework or interested to see what's on the other
side.

If you're in the latter camp, you can check out the 
[source of this website](https://github.com/bahlo/arne.me) to get some 
inspiration.
But most importantly: Choose a tech stack that excites you and have fun!

[^1]: Static Site Generators, in case you didn't make that connection yet.
[^2]: Heresy!
[^3]: If I would've chosen Go, I probably would've looked at [goldmark](https://github.com/yuin/goldmark), [goldmark-frontmatter](https://github.com/abhinav/goldmark-frontmatter), [html/template](https://pkg.go.dev/html/template) and [encoding/xml](https://pkg.go.dev/encoding/xml).
