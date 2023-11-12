---
title: "Archive Your Old Projects"
description: "In this post I describe how I wish I had archived all my old projects and my approach going forward."
published: "2023-11-12"
location: "Frankfurt, Germany"
---

Yesterday, while looking through a folder called _old things lol glhf_, I fell 
into a rabbit hole of old abandoned projects—mostly websites and graphic 
design, but I also found one or two Flash[^1] projects and compiled `.exe` 
files[^2].

And, while it was really fun remembering projects I've long forgotten, there
was no structure, and it was often difficult to figure out what a project did 
and what it looked like—some even missed crucial data.
This made me think about how I want to archive my projects going forward. 

<!-- more -->

Here is my new strategy:

## Leave it online

If the project is on the web, doesn't require maintenance and doesn't cost you
money, leave it online.

Occasionally, you'll need to move to a different domain, for example when re-doing
a website.
When talking to [Ollie](https://flbn.sh) about this, he told me that some people 
leave their old websites online at `<year>.<domain>` and I love that idea[^3]. 

You can look up old content and redirect links, so your
[URIs stay cool](https://www.w3.org/Provider/Style/URI).
And in ten years it'll probably still be online.

## Archive it

If you can't leave it online, save it to your file system.
You don't have to go all [Johnny.Decimal](https://johnnydecimal.com), but at 
least create a dedicated folder and subfolders for every year.

But instead of just copying your project to the archive folder and be done with
it, consider these points to make life easier for your future self:

Make screenshots

: Having a screenshot allows you to relive your memories more easily without
going through the hassle of setting up a project. 
If you want to go the extra mile, do a screen recording showcasing your 
project—this has the bonus effect of hearing your voice from years ago, and it 
has more context.

Add a README

: Explain what the project did, when it was created and abandoned, who 
contributed and how to get it running again.

Back up the database

: Some of my projects are missing a database dump, so all the actual content is 
gone. Run `sqldump` or whatever export functionality your database supports
and add it next to your files.

Keep generated assets

: When using static site generators, add the folder containing the built HTML, 
CSS & JS to the archive. 
That way, all you have to do is run a static file server to be able to browse 
the complete website.

## Save it to the Internet Archive

If you don't own your platform (maybe you're publishing to Substack or Notion),
you can at least save your website to the 
[Wayback Machine](https://web.archive.org).
I would also advise saving your content somewhere you control.

## Show me yours

So that's it, that's my new project archival strategy.

How are you archiving your projects?
Am I missing anything?
[Let me know.](/contact)

[^1]: If you don't know what this is, [Wikipedia's got your back](https://en.wikipedia.org/wiki/Adobe_Flash)
[^2]: Not sure what to do with these besides deleting.
[^3]: You can find the previous version of this very website at [2023.arne.me](https://2023.arne.me).
