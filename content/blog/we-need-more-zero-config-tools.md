---
title: "We need more zero config tools"
published: "2024-10-01"
location: "Frankfurt, Germany"
description: |
  Recently, I've become fond of tools that just work, out of the box.
  This blogpost is an ode to them.
---

> It just works. &mdash; Steve Jobs

If you follow this blog, you'll know that I'm doing a series called "Emacs
Config From Scratch"[^1].
Emacs is an ~editor~ operating system, where you can configure and customize
literally everything.
I like that you can truly make it yours, but it's a lot of work to get there.

Recently, I've become fond of (command line) tools that _just work_, out of the
box[^2].
This blogpost is an ode to them.

## Fish

Julia Evans recently posted [Reasons I still love the fish shell](https://jvns.ca/blog/2024/09/12/reasons-i--still--love-fish/),
and the first point she makes is "no configuration".

Things that require plugins and lots of code in shells like ZSH, like
autosuggestions, are included and configured by default in fish.
At the time of writing this, [my fish config](https://github.com/bahlo/dotfiles/blob/87fbba772f95188f55201e6717cc8fb70ee6ac38/fish/config.fish)
has less than 31 loc, most of which are abbreviations.

I have two fish plugins configured: [z](https://github.com/jethrokuan/z) for
jumping to a directory, and [hydro](https://github.com/jorgebucaran/hydro) as
my shell prompt. Neither need configuration.

## Helix

[My Neovim config](https://github.com/bahlo/dotfiles/tree/8df1cdd47c1907a471fefaf4c798f423c6b6edf3/nvim)
had 21 external plugins.
Making LSP, tree-sitter and formatting work took a while (LSP alone needs 3
plugins) and in the end there were still things that didn't work.

I've switched to [Helix](https://helix-editor.com), which can do so much out of
the box, here's a non-exhaustive list:

* LSP (including autocompletion, show signature, go to definition, show references, etc.) just works
* Tree-sitter is built in, you can even do selections on tree-sitter objects
* A file picker and global search
* Pressing a key in normal mode shows subsequent keys you can press, and what
  they do
* You can jump to any visible word, add/remove/replace quotes or other characters
* ... and so much more

The config for the code editor I use all day is 5 loc. Here it is:

```toml
theme = "kanagawa"

[editor]
line-number = "relative"
cursorline = true
rulers = [80]
```

I will say that it takes some getting used to as it folows the `selection ->
action` model, i.e. you need to run `wd` instead of `dw` to delete the next
word.

## Lazygit

After raving about [Magit](https://magit.vc) in London, my team showed me
[Lazygit](https://github.com/jesseduffield/lazygit) and I've been using it ever
since&mdash;it's really good and it does exactly what you want it to do, without
configuration[^3].

You can toggle different screen modes, panes adjust in size when active and
pretty much everything you want to do is only a few keystrokes away.

## Zellij

A batteries-included Tmux alternative, [Zellij](https://zellij.dev) doesn't
need any configuration to work well. You can set up Layouts without additional
plugins (although there is a plugin system) and I'm generally not missing
anything from my Tmux configuration.

My favorite feature is the floating panes. Press `Ctrl + p, w` to toggle a
pane floating on top of everything else—I often use this for Lazygit.

## What else?

Do you have a tool that requires no (or minimal) configuration? 
[Send me an email](mailto:hey@arne.me) and I'll add it here!

And if you're building something, please strive to make the default experience
work really well for most people.

[^1]: [Check](/blog/emacs-from-scratch-part-one-foundations)
      [it](/blog/emacs-from-scratch-part-two)
      [out](/blog/emacs-config-from-scratch-part-three)
[^2]: I'm starting to feel the same about programming languages and external
      dependencies, but that's a different post.
[^3]: You _can_ configure [almost everything](https://github.com/jesseduffield/lazygit/blob/master/docs/Config.md)&mdash;but
      you don't need to.
