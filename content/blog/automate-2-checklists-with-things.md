---
title: "Automate #2: Checklists with Things"
description: "How to use checklists in TaskPaper-Format with Things."
published: "2019-02-09"
updated: "2023-11-11"
location: "Frankfurt, Germany"
---

This is the second post of my series _Automate_.

On the [Cortex podcast](https://www.relay.fm/cortex) (which inspired the whole series), CGP Grey and Myke Hurley sometimes talk about their checklists; whole projects that can be invoked by a tap if needed.
These lists are for things that are important to get right, but you do them not often enough to remember every step, examples are an Airport or a YouTube checklist.
They mostly use [OmniFocus](https://www.omnigroup.com/omnifocus) for this, which can export and import projects as [TaskPaper](https://taskpaper.com/).

<!-- more -->

When thinking about this, I found more and more usecases for checklists and really wanted to have this set up, but I personally can't deal with the way some things work in OmniFocus.
I use [Things](https://culturedcode.com/) for my todos, which doesn't support TaskPaper, but supports its own [JSON format](https://support.culturedcode.com/customer/en/portal/articles/2803573).
I didn't want to store my checklists in a proprietary format like this Things-specific JSON, I wanted TaskPaper.

Then I remembered playing around with [Scriptable](https://scriptable.app/), which could execute JavaScript on iOS with native bindings to UI, Clipboard, etc. – this sounded perfect for my usecase, so I started writing a script to convert TaskPaper to Things JSON, which will be the core of my checklist flow.

## The Script

I ended up writing a 300-line script ([it's on GitHub](https://github.com/bahlo/scriptable-scripts/blob/master/TaskPaperToThings.js)) with only enough functionality to fit my needs (e.g. it only supports tabbed indentation and may break on the slightest deviation).

Since I wanted to run this script from Apples Shortcuts app, I needed to use a hack for input/output data.
Many apps face these problems and use the clipboard (e.g. [AutoSleep](https://autosleep.tantsissa.com/shortcutsapp#TOC-How-do-I-use-AutoSleep-dictionaries-in-the-shortcuts-app-)), so that's what I did as well.
If you run the script in the Scriptable app, it will get the clipboard contents expecting text in TaskPaper format, convert it to the Things JSON and copy it.

Here's how the `@tag` and `@attr(value)` work:

- `@due(2019-02-09)` would be setting the due date
- `@today` will set the task or project to start today
- `@start(2019-02-09)` will set the task or project to start on the given date (in `YYYY-MM-DD` format)
- `@due(2019-02-09)` will set the due-date of a task or project to the given date
- Anything else without a value (like `@foo`) will be a Things tag (this tag has to exist beforehand)
- Anything else with a value will be a Things attribute, see the [reference](https://support.culturedcode.com/customer/en/portal/articles/2803573) for attributes

## The Shortcut

I wanted one shortcut to start all checklists from. Sadly you cannot get a directory listing from an iCloud folder, so for now if I add a new checklist, I have to add the filename to the `List` at the top.

This is what the Shortcut looks like:

<picture>
  <source srcset="/blog/automate-2-checklists-with-things/shortcut.avif" type="image/avif" />
  <img src="/blog/automate-2-checklists-with-things/shortcut.png" alt="An iOS shortcut: Select from a list of items, get the file from iCloud Drive, copy it to the clipboard, run the Scriptable script, get the clipboard, url encode it and open Things using the URL scheme" />
</picture>

After a checklist was chosen, the shortcut gets the file from iCloud Drive (it has to be in the Shortcuts application folder to be accessible).
Then it copies the file contents to the Clipboard and runs the script (the Scriptable action should appear automatically in your Shortcuts app once you created it). After that, it retrieves the Things JSON from the clipboard, url-encodes it and opens Things with its URL scheme.

## Conclusion

This shortcut-scriptable-combination helps to use checklists in Things from anywhere while storing them in a highly-compatible format. I really love to find more use cases for checklists. Do you use checklists? What usecases do you have? I'd love to [hear from you](/contact).
