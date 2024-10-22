---
title: "Jujutsu in practice"
description: |
  Jujutsu is a version control system, an alternative to Git. This blog post
  is not a tutorial, but practical examples of how I use it.
published: "2024-10-21"
location: "Frankfurt, Germany"
lobsters: "https://lobste.rs/s/fbjowx/jujutsu_practice"
---

This post is not about the Japanese martial arts _Jiu-jitsu_, it's about a new
VCS, or version control system.
There are some great tutorials and introductions for
[Jujutsu](https://github.com/martinvonz/jj), short `jj`, so I want to give some
insight in how I use it day to day for this website.

## Initialize a repository

You can initialize `jj` in an existing `git` repository like this:

```sh
$ jj git init --git-repo .
```

This will create a `.jj` directory next to `.git`.
You can now use both `git` and `jj`, although I wouldn't recommend it.
There's a work-in-progress native storage backend, but since all my projects
are `git` repositories anyway, this works great for me.

Note that this is non-destructive; if you want to go back to `git`, all it takes
is a `rm -r .jj`.

## Get an overview

Running `jj log` in the repository for this very website gives this output:

```sh
$ jj log
@  qzsvtxpv hey@arne.me 2024-10-21 09:58:06 e18f7532
│  Add blog/jj-in-practice
│ ○  yoxxsupn hey@arne.me 2024-10-20 22:55:03 ae5d9109
├─╯  Add library/calibans-war
│ ○  tvkvwslw hey@arne.me 2024-10-20 22:49:54 5e4dee1f
├─╯  Add library/the-posthumous-memoirs-of-bras-cubas
│ ○  pywmtrys hey@arne.me 2024-10-20 21:20:11 7bda14b7
│ │  Add atoms/1
│ ○  xnlzypwn hey@arne.me 2024-10-20 21:20:10 8a004404
├─╯  Add atoms functionality
◆  wxxtrmqk hey@arne.me 2024-10-20 16:18:15 main HEAD@git 1eb46c81
│  Add weekly/166
~
```

This already shows one of the biggest differences, compared to `git`:
There's no branches, other than `main`.
You _can_ create branches, which are called _bookmarks_ in `jj`, but you don't
_need_ to.
Instead, you work mostly with changes[^1].

The terminal above shows the change `w` (you can use the first letter to
reference changes, on your terminal it'll be highlighted as well) as a parent
to `x`, `t`, `y` and `q`.
All these child-revisions don't have a branch/bookmark, but they don't need one.
You can see what's in-flight at this repository better than with any `git` repo,
especially if branches haven't been cleaned up in a while.

## Create a revision

My usual flow with `git`, is to leave chages in the staging area until I'm
ready to commit.
Sometimes, if I have to switch branches or want to save my work, I'll stash
or create a WIP commit. 

In `jj`, there is no staging area—everything is a revision.
Let's create a new revision on top of my revisions where I add atoms
functionality:

```sh
$ jj new -r p
```

Running `git log` again:

```sh
$ jj log
@  kxqvnxnw hey@arne.me 2024-10-21 10:03:20 22c020cf
│  (empty) (no description set)
○  pywmtrys hey@arne.me 2024-10-20 21:20:11 HEAD@git 7bda14b7
│  Add atoms/1
○  xnlzypwn hey@arne.me 2024-10-20 21:20:10 8a004404
│  Add atoms functionality
│ ○  qzsvtxpv hey@arne.me 2024-10-21 10:03:18 27229dca
├─╯  Add blog/jj-in-practice
│ ○  yoxxsupn hey@arne.me 2024-10-20 22:55:03 ae5d9109
├─╯  Add library/calibans-war
│ ○  tvkvwslw hey@arne.me 2024-10-20 22:49:54 5e4dee1f
├─╯  Add library/the-posthumous-memoirs-of-bras-cubas
◆  wxxtrmqk hey@arne.me 2024-10-20 16:18:15 main 1eb46c81
│  Add weekly/166
~
```

You'll notice that our active revisions are now left-aligned, and the one to
add this very blog post has moved to the right. 
There's no hirarchy, they're all descendants of `Add weekly/166`.

After doing some work, e.g. addings a new atom, I can _describe_ that revision
with `jj describe`.
This is comparable to `git commit`, but it doesn't actually create a commit or
a revision, it only _describes_ the current one.

Sometimes I want to update a previous revision, in this case `Add atoms/1`.
I can run `jj squash` to merge the current one with its parent.

## Push and pull

To fetch new revisions, I run `jj git fetch`, to push branches/bookmarks, I run
`jj git push`.
This uses the same `git` server it was using before.

Before pushing, I need to move my bookmark to the revision I want to push.
I push the `main` branch to deploy my website, so if I wanted to publish my
atoms functionality (should I?), I would run `jj bookmark set main -r p` before
pushing.

## Rebase and split

Sometimes I need to rebase. Fortunately that's a lot simpler than it is in
`git`:
I can run `jj rebase -s <source> -d <destination>` to move revisions around.
If I wanted support for atoms for this blog post, I would run
`jj rebase -s q -d p` and it would move the revision for this blog post on top
of "Add atoms/1".

`jj` also does automatic rebasing, e.g. if you squash changes into a revision
that has descendants.

And if I have a revision that I'd like to be two, I run `jj split` and 
interactively select what belongs to which revision.

## Undo

Undoing an (interactive) rebase in `git` is not fun. 
`jj undo` undoes the last `jj` operation, doesn't matter if it's abandoning
(deleting) a revision or doing a rebase.
This is a life saver!
You can also run `jj op log` to display your last `jj` operations.

## Things I stumble upon

~I've been using `git` for a long, long time.
My brain assumes that after a `commit`, I'm working on the next one.
It also assumes that `jj describe` does the same as `git commit` (it's not).
I often describe a revision and continue editing files, which then erroneously
get added to the current revision.
I'm not saying this is wrong, it makes sense in the `jj` world, but I keep
tripping over that and have to run `jj split` to pull changes out again.~
<br>
alterae on Lobste.rs [pointed out](https://lobste.rs/s/fbjowx/jujutsu_practice#c_xyhzxa)
that you can describe and immediatly create a new revision on top of it with
`jj commit`. Thanks!

~One other thing is that you cannot check out a revision directly (or maybe I
just don't know how to), so when I've moved to a different revision and want
to move back, I run `jj new <revision>`, which creates an empty revision on top
of it.
This means that if I'm not done with the revision, I have to keep running
`jj squash` to move changes into it.~
<br>
gecko on Lobste.rs [pointed out](https://lobste.rs/s/fbjowx/jujutsu_practice#c_ytil6w)
that you can check out a revision directly with `jj edit <revision>`. Thanks!

## Why it works for me

A week ago, I removed `jj` from my website's repository, to see if I'd miss it.
I added it back the same day.
Jujutsu feels _lighter_ than `git`, while at the same time giving you a full
overview of what's in-flight right now[^2].
Having no staging area means I only need to worry about revisions (see caveat
above).

If trying new things sounds fun to you, give 
[Jujutsu](https://github.com/martinvonz/jj) a spin!

## Further reading

* [Official Jujutsu Tutorial](https://martinvonz.github.io/jj/v0.13.0/tutorial/)
* [Comparison with Git](https://martinvonz.github.io/jj/latest/git-comparison/)
* [Steve's Jujutsu Tutorial](https://steveklabnik.github.io/jujutsu-tutorial/)
* [Chris Krycho's jj init essay](https://v5.chriskrycho.com/essays/jj-init/)
* [Chris Krycho's video on jj](https://www.youtube.com/watch?v=2otjrTzRfVk)

[^1]: What's cool about a `jj` change, is that updating it doesn't change it's ID.
[^2]: If you work in large projects with many contributors, you can 
      [tune your `jj log` to only your revisions](https://martinvonz.github.io/jj/latest/tutorial/#the-log-command-and-revsets).
