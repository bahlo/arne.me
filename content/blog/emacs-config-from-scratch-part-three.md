---
title: "Emacs Config From Scratch, Part 3: LSP & Tree-sitter"
description: "In this third part of my Emacs Config From Scratch series, I configure LSP and Tree-sitter."
published: "2024-05-11"
location: "Frankfurt, Germany"
---

This is Part 3 of my series, _Emacs Config From Scratch_[^1], where I create
my perfect editor using Emacs.
In this post, I'll do some housekeeping, set up LSP[^2], language modes for Rust,
Go, TypeScript and Zig, and add search.

<!-- more -->

## Table Of Contents

* [Housekeeping](#housekeeping)
* [LSP](#lsp)
* [Tree-sitter](#tree-sitter)
* [Language support](#language-support)
* [Search](#search)
* [Wrapping up](#wrapping-up)

## Housekeeping

The first thing I want to do is make the UI titlebar match the Emacs theme and 
hide the file icon:

```lisp
(use-package emacs
  :init
  (add-to-list 'default-frame-alist '(ns-transparent-titlebar . t))
  (add-to-list 'default-frame-alist '(ns-appearance . light))
  (setq ns-use-proxy-icon  nil)
  (setq frame-title-format nil))
```

The next thing is automatically loading `$PATH` from my shell using 
[exec-path-from-shell](https://github.com/purcell/exec-path-from-shell)—this 
is important so Emacs can find our installed binaries, e.g. for language servers:

```lisp
(use-package exec-path-from-shell
  :init
  (exec-path-from-shell-initialize))
```

Another problem I ran into was writing square brackets when on the MacBook, as it
would interpret the keys as `Meta-5`/`Meta-6`.
I fixed that by updating the keybindings from 
[Part 1](/blog/emacs-from-scratch-part-one-foundations):

```lisp
(use-package emacs
  :init
  (when (eq system-type 'darwin)
    (setq mac-command-modifier 'super)
    (setq mac-option-modifier nil)
    (setq mac-control-modifier nil)))
```

I like to keep most of my code at 80 characters, so let's add a ruler:

```lisp
(use-package emacs
  :init
  (setq-default fill-column 80)
  (set-face-attribute 'fill-column-indicator nil
                      :foreground "#717C7C" ; katana-gray
                      :background "transparent")
  (global-display-fill-column-indicator-mode 1))
```

Finally, we want to store backup files in `~/.saves` instead of next to the file
we're saving:

```lisp
(use-package emacs
  :config
  (setq backup-directory-alist `(("." . "~/.saves"))))
```

## LSP

Let's install [company-mode](https://company-mode.github.io) first, for 
auto-completion:

```lisp
(use-package company-mode
  :init
  (global-company-mode))
```

Now we configure the built-in LSP package 
[eglot](https://github.com/joaotavora/eglot):

```lisp
(use-package emacs
  :hook (zig-mode . eglot-ensure)
  :hook (rust-mode . eglot-ensure)
  :hook (go-mode . eglot-ensure)
  :hook (typescript-mode . eglot-ensure)
  :general
  (leader-keys
    "l" '(:ignore t :which-key "lsp")
    "l <escape>" '(keyboard-escape-quit :which-key t)
    "l r" '(eglot-rename :which-key "rename")
    "l a" '(eglot-code-actions :which-key "code actions")))
```

This runs `eglot-ensure` in languages we have language servers installed for.
It also sets up `SPC l r` to rename a symbol and `SPC l a` to prompt for code 
actions.

## Tree-sitter

We'll use [treesit-auto](https://github.com/renzmann/treesit-auto) to 
automatically install and use tree-sitter major modes:

```lisp
(use-package treesit-auto
  :custom
  (treesit-auto-install 'prompt)
  :config
  (treesit-auto-add-to-auto-mode-alist 'all)
  (global-treesit-auto-mode))
```

This is handy because it doesn't require us to think about using e.g. 
`zig-ts-mode` instead of `zig-mode`, it handles everything for us.

## Language support

Next, we install all language modes we need:

```lisp
(use-package markdown-mode
  :config
  (setq markdown-fontify-code-blocks-natively t))
(use-package zig-mode
  :general
  (leader-keys
    "m" '(:ignore t :which-key "mode")
    "m <escape>" '(keyboard-escape-quit :which-key t)
    "m b" '(zig-compile :which-key "build")
    "m r" '(zig-run :which-key "run")
    "m t" '(zig-test :which-key "test")))
(use-package rust-mode
  :general
  (leader-keys
    "m" '(:ignore t :which-key "mode")
    "m <escape>" '(keyboard-escape-quit :which-key t)
    "m b" '(rust-compile :which-key "build")
    "m r" '(rust-run :which-key "run")
    "m t" '(rust-test :which-key "test")
    "m k" '(rust-check :which-key "check")
    "m c" '(rust-run-clippy :which-key "clippy")))
(use-package go-mode)
(use-package gotest
  :general
  (leader-keys
    "m" '(:ignore t :which-key "mode")
    "m <escape>" '(keyboard-escape-quit :which-key t)
    "m t" '(go-test-current-project :which-key "test")
    "m r" '(go-run :which-key "run")))
(use-package typescript-mode)
```

I'm using `SPC m` to change based on major-mode, e.g. means `SPC m t` means test
in most programming modes, but won't exist in `markdown-mode`.

## Search

Sometimes we don't know what file we're looking for, so let's add 
[rg.el](https://github.com/dajva/rg.el) to help us find it:

```lisp
(use-package rg
  :general
  (leader-keys
    "f" '(rg-menu :which-key "find")))
```

This opens a Magit-like menu and allows you to search in various modes (dwim, 
regex, literal, etc.).

## Wrapping up

Opening a Zig project now looks like this[^3]; see also the final
[`init.el`](/blog/emacs-config-from-scratch-part-three/init.el):

![A screenshot of Emacs with a dark theme showing Zig code and a context menu for code actions](/blog/emacs-config-from-scratch-part-three/zig-lsp.png)


I'm going to switch to Emacs as my primary editor and tune it further in the coming
weeks. 
In the next part, I want to add support for Org-mode, show a dashboard on startup,
enable font ligatures and fix all the small things that I'll find.

Subscribe to the [RSS feed](/blog/atom.xml) so you don’t miss Part 4, 
and [let me know](mailto:hey@arne.me) what you think!

[^1]: Check out parts [one](/blog/emacs-from-scratch-part-one-foundations) and
      [two](/blog/emacs-from-scratch-part-two) if you haven't already!
[^2]: [Language Server Protocol](https://en.wikipedia.org/wiki/Language_Server_Protocol)
[^3]: By the way, I switched my theme to 
      [Kanagawa](https://github.com/meritamen/emacs-kanagawa-theme). 
