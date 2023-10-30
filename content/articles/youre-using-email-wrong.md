---
title: "You’re Using Email Wrong"
description: "If you don’t like email, try a different strategy."
published: "2022-03-06"
updated: "2022-03-08"
---

You probably don't like email, not a lot of people do.
That's because you're using it wrong.

Chances are that if you look at your inbox, it's full of unsolicited marketing
emails, log-in notifications or spam.
Or you're doing inbox zero and all that trash lives in your archive.

As everything else, email is subject (hah) to entropy.
If you're not careful, chaos will take over and your email inbox will look like
the screenshot above.

## A different concept

As controversial as the company behind [HEY](https://www.hey.com) is, the
concept that they introduced with their mail app has fundamentally changed how I
think about email.
I adapted the part that resonated with me to my [Fastmail](https://fastmail.com)
account like this:

### Inbox

This is where all emails sent by humans end up.
That's it.

### Papertrail

Notifications, invoices, everything that you don't want to delete but are not
really interested in.
This folder has about 1.6k unread emails right now.
They're not meant to be read, but if I need to look something up, I know where
to look.

### Newsfeed

I'm subscribed to over 20 newsletters and all of them end up in here.
If I have time, I'll read the newsletters and add the articles I want to read to
[Instapaper](https://instapaper.com).

By the way, if you're looking for something to add to your newsfeed, check out
[Arne's Weekly](/weekly).

## The Setup

I'm using Fastmail with a custom domain and have aliases with rules for the
different destinations, for example `papertrail@example.org` or
`newsfeed@example.org`.

This way, if I sign up for a new service or subscribe to a newsletter, instead
of having to adapt existing rules, I can use the proper email address and
everything will go where it should.

This is not Fastmail specific, you can do the same with most providers.
For example, if you have `hikingfan@gmail.com`, you could set a rule for
`hikingfan+papertrail@gmail.com` and one for `hikingfan+newsfeed@gmail.com` and
use those email addresses when signing up.

## The human factor

If I look at my inbox, it's a joy.
Feedback to blog posts, articles and personal messages from people I care about.
Sticking with this strategy made email about humans, not about machines.

I encourage you to try it!

If you have a different concept, feedback or ideas, please
[let me know](mailto:hey@arne.me).

**Update (Mar 08, 2022):** Multiple people have responded with their systems and
questions (which I love, please keep doing that), some missed a way to track if
a service "lost" their email address.
If you use an alias, e.g. `papertrail@example.org`, you can use the `+` operator
like this: `papertrail+twitter@example.org`.
The sorting rule will be a bit more complicated, but you won't have to adapt
it every time you sign up for something.
