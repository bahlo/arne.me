---
title: ".vim-project"
description: "This post explains how to get project-specific Vim settings."
published: "2014-08-14"
location: "Wetteraukreis, Germany"
---

If you're using Vim, you know that feel (if you aren't, you can skip this
article): Everytime you open a project, you toggle
[NERDTREE](https://github.com/scrooloose/nerdtree) and
[Tagbar](https://github.com/majutsushi/tagbar) (or similar). But you don't
want to put that in your `.vimrc`, because then they'd open every time, even
when you just want to quickly edit a file.

<!-- more -->

## But wait

Sublime Text does have a `<name>.sublime-project`, which holds configuration
for the curent project. Why doesn't Vim has something like that?

## Just do it yourself
It's not hard, paste the following lines anywhere in your `.vimrc`:

```vim
if filereadable(expand(".vim-project"))
  source .vim-project
endif
```

You can now create a `.vim-project`-file anywhere you want and just write
stuff in it like in your `.vimrc`, this is one of mine:

```vim
autocmd VimEnter * NERDTree
autocmd VimEnter * Tagbar
```

_Note: You may want to close Vim automatically after all windows but NERDTree
are closed, see the [FAQs](https://github.com/scrooloose/nerdtree#faq) for
that._
