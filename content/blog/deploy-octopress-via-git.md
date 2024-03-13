---
title: "Deploy Octopress, the right way"
description: "This post explains how to deploy an Octopress page via Git"
published: "2013-09-28"
location: "Wetteraukreis, Germany"
---

You're deploying your Octopress blog via Git to GitHub Pages (or Heroku), but
you don't like Heroku and GitHub Pages are refreshing too slow and you really
don't want to use Rsync, do you?

Just deploy to your own server via Git.

<!-- more -->

## Step one

If you haven't already done this, run `rake setup_github_pages` in your local Octopress repository and give it any valid GitHub url.

## Step two

Set up a bare Git repository on your server to push to

```sh
mkdir -p ~/projects/octopress.git
cd ~/projects/octopress.git
git init --bare # Related: http://git-scm.com/book/ch4-2.html
```

### Step two and a half

Create a `post-receive` on your server in `~/projects/octopress.git/hooks/` and give yourself execute permissions with `chmod u+x post-receive`.

```sh
#!/bin/bash
# You already know that you'll have to change the paths
git --work-tree=$HOME/html/octopress --git-dir=$HOME/projects/octopress.git checkout -f
```

## Step three
Back in your local Octopress installation.

```
cd _deploy
git remote set-url origin user@domain.com:projects/octopress.git
git pull -u origin master # Where master is the branch you set in your Rakefile as deploy_branch, defaults to master
```

And you're set. If you run into any trouble, let me know.
