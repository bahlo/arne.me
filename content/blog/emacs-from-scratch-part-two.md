---
title: "Emacs Config From Scratch, Part Two: Projects and Keybindings"
published: "2023-12-28"
location: "Frankfurt, Germany"
description: |
  In this second post in my Emacs from Scratch series, we’ll
  set up a way to manage projects, quickly find files, set up
  custom keybindings, interact with Git and open a terminal inside Emacs.
---

This is the second post in my Emacs From Scratch series.

In [Part 1](/blog/emacs-from-scratch-part-one-foundations), we've set up
the UI and evil mode.
Today we'll set up a way to manage projects, quickly find files, set up custom keybindings, interact with Git and open a terminal inside Emacs.

<!-- more -->

## Table of Contents

- [Project manager](#project-manager)
- [Custom keybindings](#custom-keybindings)
- [Fuzzy finding](#fuzzy-finding)
- [Git integration](#git-integration)
- [Terminal](#terminal)
- [Small tweaks](#small-tweaks)
  - [Make `Ctrl-u` work like in Vim](#make-ctrl-u-work-like-in-vim)
  - [Comment lines](#comment-lines)
  - [Optimizing the garbage collector](#optimizing-the-garbage-collector)
  - [Don't use ESC as a modifier](#dont-use-esc-as-a-modifier)
- [Conclusion](#conclusion)

## Project manager

Vim is usually terminal-first; you navigate to a directory and open Vim.
Emacs is the other way around; you start Emacs, open a project and maybe a terminal buffer (see [Terminal](#terminal) further down).

Let's set up [Projectile](https://github.com/bbatsov/projectile) to manage our
projects and quickly find files.

```lisp
(use-package projectile
  :demand
  :init
  (projectile-mode +1))
```

You can now run `M-x` (a.k.a. `Opt-x` on macOS) and type
`projectile-add-known-project` to add a project as well as
`projectile-switch-project` to open a project.

This is neither fast, nor discoverable. Let's set up some custom keybindings.

## Custom keybindings

Before we define our own keybindings, we need to do something to improve
discoverability. [which-key](https://github.com/justbur/emacs-which-key) will
show available commands as you start a keybinding sequence:

```lisp
(use-package which-key
  :demand
  :init
  (setq which-key-idle-delay 0.5) ; Open after .5s instead of 1s
  :config
  (which-key-mode))
```

We'll use [general.el](https://github.com/noctuid/general.el) because it makes
it super easy to define keybindings and allows us to define them in a
`use-package` function.

We'll have `SPC` as our leader key, which allows us to press `SPC` and have all our custom keybindings show up.

```lisp
(use-package general
  :demand
  :config
  (general-evil-setup)

  (general-create-definer leader-keys
    :states '(normal insert visual emacs)
    :keymaps 'override
    :prefix "SPC"
    :global-prefix "C-SPC")

  (leader-keys
    "x" '(execute-extended-command :which-key "execute command")
    "r" '(restart-emacs :which-key "restart emacs")
    "i" '((lambda () (interactive) (find-file user-init-file)) :which-key "open init file")

    ;; Buffer
    "b" '(:ignore t :which-key "buffer")
    ;; Don't show an error because SPC b ESC is undefined, just abort
    "b <escape>" '(keyboard-escape-quit :which-key t)
    "bd"  'kill-current-buffer
  )
```

We're using `:which-key` to add a description that will show up next to the
command.
To add custom keybindings to Projectile, we need to edit the `use-package`
definition and move it after the `use-package general` function:

```lisp
(use-package projectile
  :demand
  :general
  (leader-keys
    :states 'normal
    "SPC" '(projectile-find-file :which-key "find file")

    ;; Buffers
    "b b" '(projectile-switch-to-buffer :which-key "switch buffer")

    ;; Projects
    "p" '(:ignore t :which-key "projects")
    "p <escape>" '(keyboard-escape-quit :which-key t)
    "p p" '(projectile-switch-project :which-key "switch project")
    "p a" '(projectile-add-known-project :which-key "add project")
    "p r" '(projectile-remove-known-project :which-key "remove project"))
  :init
  (projectile-mode +1))
```

Now we can press `SPC` and get suggestions that we can navigate along.
`SPC SPC` lets you open a file in the current project (or a project if none is open),
`SPC b` opens buffer options, `SPC p` project options, etc.

This is what it looks like:

<picture>
  <source srcset="/blog/emacs-from-scratch-part-two/which-key.avif" type="image/avif" />
  <img src="/blog/emacs-from-scratch-part-two/which-key.avif" alt="And Emacs window with a drawer open showing different shortcuts, e.g. SPC → find file, r → +buffer et al" />
</picture>

But when you try to find a file, or add a project, you'll notice that this is
clunky as you need to type the exact path for it to work.
Let's fix that.

## Fuzzy finding

We'll be using [ivy](https://github.com/abo-abo/swiper) as our generic completion frontend.
This will make choosing files and projects a lot more ergonomic:

```lisp
(use-package ivy
  :config
  (ivy-mode))
```

Now we get a list of possible entries and live-search.

## Git integration

To manage source control, we'll use [Magit](https://github.com/magit/magit), which is–rightfully–widely considered to be the best Git client ever:

```lisp
(use-package magit
  :general
  (leader-keys
    "g" '(:ignore t :which-key "git")
    "g <escape>" '(keyboard-escape-quit :which-key t)
    "g g" '(magit-status :which-key "status")
    "g l" '(magit-log :which-key "log"))
  (general-nmap
    "<escape>" #'transient-quit-one))
```

To make Magit work nicely with Evil, let's add [evil-collection](https://github.com/emacs-evil/evil-collection):

```lisp
(use-package evil-collection
  :after evil
  :demand
  :config
  (evil-collection-init))
```

In addition we need to add this to the `:init` block of `use-package evil` to
prevent evil and evil-collection interfering:

```lisp
(setq evil-want-keybinding nil)
```

Now `SPC g g` will open up the current Git status.
You can stage single files with `s`, all files with `S` and commit by pressing `cc`.

Finally, we want to highlight uncommited changes in the gutter:

```lisp
(use-package diff-hl
  :init
  (add-hook 'magit-pre-refresh-hook 'diff-hl-magit-pre-refresh)
  (add-hook 'magit-post-refresh-hook 'diff-hl-magit-post-refresh)
  :config
  (global-diff-hl-mode))
```

## Terminal

Even though we'll try to make everything work from inside Emacs, sometimes it's just quicker and easier to have a shell.

We'll use [vterm](https://github.com/akermu/emacs-libvterm) as a terminal emulator we can use inside emacs.

```lisp
(use-package vterm)
```

We'll also install [vterm-toggle](https://github.com/jixiuf/vterm-toggle), a package that allows us to toggle between the active buffer and a vterm buffer:

```lisp
(use-package vterm-toggle
  :general
  (leader-keys
    "'" '(vterm-toggle :which-key "terminal")))
```

Pressing `SPC '` will open a terminal. Pressing it again will hide it, but keep any processes running.

## Small tweaks

As every time, we'll do a few small tweaks:

### Make `Ctrl-u` work like in Vim

Add the following to the `:init` block of `use-package evil`:

```lisp
(setq evil-want-C-u-scroll t)
```

### Comment lines

We want similar behaviour to [commentary.vim](https://github.com/tpope/vim-commentary) and comment objects in/out with `gc`:

```lisp
(use-package evil-nerd-commenter
  :general
  (general-nvmap
    "gc" 'evilnc-comment-operator))
```

### Optimizing the garbage collector

We'll use [GCMH](https://github.com/emacsmirror/gcmh), "the Garbage Collector Magic Hack", to minimize GC interference with user activity:

```lisp
(use-package gcmh
  :demand
  :config
  (gcmh-mode 1))
```

Move this up to be the first package loaded after configuring `use-package`, to improve start-up time.

### Don't use ESC as a modifier

If you want to exit a menu, `<escape>` is the key of choice, esp. coming from Vim. Let's match that behaviour:

```lisp
(use-package emacs
  :init
	(global-set-key (kbd "<escape>") 'keyboard-escape-quit))
```

## Conclusion

We now have everything we need to manage projects, navigate to files, run terminal commands and manage Git.
Starting with this post, I'm using this very setup to edit this series.

In part 3, we'll set up Tree-sitter and, if available, LSP for Rust, Go, TypeScript and Markdown.

Subscribe to the [RSS Feed](/blog/atom.xml) so you don't miss the following
parts, and [let me know](/contact) what you think!
