---
title: "Plex on NixOS"
description: "In this post I describe how I set up Plex on NixOS, including a virtual file system for Backblaze B2 and Nginx for HTTPS."
published: "2022-02-22"
location: "Frankfurt, Germany"
---

A few weeks ago, the hard drive (yes, I know) in my home lab died.
It was a sad moment, especially because I ran Plex on it and rely on that for my
music and audiobook needs.

The upside is that it gave me the opportunity to rethink my Plex setup.
Hosting it at home is great for storage costs and control, but it's hard to
share with friends or access on the go, especially with a NATed IPv4, so I
decided to move to the cloud.

<!-- more -->

## Table of Contents

## Choosing a server and storage method

I chose [Hetzner Cloud](https://cloud.hetzner.com) because I like their service,
and they use green energy.

The biggest challenge was storage. Hetzner charges around €50/month for a 1 TB
volume (others have comparable pricing).

But then my friend Eric told me about [rclone](https://rclone.org) and its
ability to mount blob storage (which is cheap) as a virtual disk.
That means Plex sees all files as if they were actually there and if it tries
to read a file, it's downloaded on demand if it's not cached already.

Armed with this knowledge, I started setting up the server.

## Setting up NixOS

NixOS is a declarative and reproducible operating system.
You have a configuration file in `/etc/nixos/configuration.nix` that defines
your installed applications, configuration and system setup.
And if you mess up, you can always roll back.

The first I did was creating a server on Hetzner with any distribution (I went
width a [CPX11](https://www.hetzner.com/cloud#pricing) and Ubuntu) and then following the instructions on the
[install scripts for Hetzner Cloud](https://github.com/nix-community/nixos-install-scripts/tree/master/hosters/hetzner-cloud).

If you follow along, make sure to choose a server with at least 40 GB of disk
space.

After booting into NixOS, I changed the root password by running `passwd` and
upgraded NixOS (see [Upgrading NixOS](https://nixos.org/manual/nixos/stable/index.html#sec-upgrading)).
If you want to further secure your NixOS installation, Christine Dodrill has a
great guide called [Paranoid NixOS Setup](https://christine.website/blog/paranoid-nixos-2021-07-18).

## Setting up storage

I decided to go with [Backblaze B2](https://www.backblaze.com/b2/cloud-storage.html)
as I have used it before, it's cheaper than S3, and I don't support Amazon.
If you want to use something else, rclone supports
[a lot of providers](https://rclone.org/#providers).

After creating a bucket for the media, I created an Application Key and made
note of the `keyID` and `applicationKey`.

Then I added the following lines to my Nix configuration at
`/etc/nixos/configuration.nix` to install rclone and create a
`/etc/rclone/rclone.conf` for the bucket:

```nix
environment.systemPackages = [ pkgs.rclone ];

environment.etc = {
  "rclone/rclone.conf" = {
    text = ''
      [b2]
      type = b2
      account = <keyID>
      key = <applicationKey>
      hard_delete = true
      versions = false
    '';
    mode = "0644";
  };
};
```

If you follow along, make sure to replace `<keyID>` and `<applicationKey>`.

By the way, NixOS comes with nano preinstalled, so if you want a real editor,
you can get it with the following command:

```shell
$ nix-shell -p vim
```

For the disk mount, I created a Systemd service that mounts the bucket on start
and automatically starts on boot.

```nix
systemd.services.plex_media = {
  enable = true;
  description = "Mount media dir";
  wantedBy = ["multi-user.target"];
  serviceConfig = {
    ExecStartPre = "/run/current-system/sw/bin/mkdir -p /mnt/media";
    ExecStart = ''
      ${pkgs.rclone}/bin/rclone mount 'b2:<bucket name>/' /mnt/media \
        --config=/etc/rclone/rclone.conf \
        --allow-other \
        --allow-non-empty \
        --log-level=INFO \
        --buffer-size=50M \
        --drive-acknowledge-abuse=true \
        --no-modtime \
        --vfs-cache-mode full \
        --vfs-cache-max-size 20G \
        --vfs-read-chunk-size=32M \
        --vfs-read-chunk-size-limit=256M
    '';
    ExecStop = "/run/wrappers/bin/fusermount -u /mnt/media";
    Type = "notify";
    Restart = "always";
    RestartSec = "10s";
    Environment = ["PATH=${pkgs.fuse}/bin:$PATH"];
  };
};
```

If you follow along, make sure to replace `<bucket name>`.

The `--vfs-*` arguments configure the virtual file system.
I only have 40 GB local disk space, so I set the cache size to 20 GB (using
`--vfs-cache-max-size`).

I then ran `nixos-rebuild switch` to apply the configuration, uploaded some
data to the bucket and listed `/mnt/media` to make sure everything works.

## Configuring Plex

NixOS has a predefined service for Plex, which I used like this:

```nix
nixpkgs.config.allowUnfree = true; # Plex is unfree

services.plex = {
  enable = true;
  dataDir = "/var/lib/plex";
  openFirewall = true;
  user = "plex";
  group = "plex";
};
```

With this configuration, Nix will open the correct ports in the firewall,
create a user called `plex` with a group also called `plex` and install the Plex
Media Server with the configuration in `/var/lib/plex`.

### Adding an Audiobooks Plugin

I wanted to use the [Audiobooks.bundle](https://github.com/macr0dev/Audiobooks.bundle)
metadata agent for better matching, so I added this to the `let`-section at the
top of `plex.nix`:

```nix
let
  audiobooksPlugin = pkgs.stdenv.mkDerivation {
    name = "Audiobooks.bundle";
    src = pkgs.fetchurl {
      url = https://github.com/macr0dev/Audiobooks.bundle/archive/9b1de6b66cd8fe11c7d27623d8579f43df9f8b86.zip;
      sha256 = "539492e3b06fca2ceb5f0cb6c5e47462d38019317b242f6f74d55c3b2d5f6e1d";
    };
    buildInputs = [ pkgs.unzip ];
    installPhase = "mkdir -p $out; cp -R * $out/";
  };
in
  # ...
```

That fetches the commit `9b1de6b` of the audiobooks plugin and makes sure that
the SHA256 is correct.

Then I told Plex to use this plugin like this:

```nix
services.plex.managePlugins = true;
services.plex.extraPlugins = [audiobooksPlugin];
```

If you're following along and get an error which says
`services.plex.managePlugins` no longer has an effect, remove that line.

At this point, after running `nixos-rebuild switch` again, I was able to access
the Plex interface at `https://<domain or ip>:32400`.

Plex needs an initial configuration, but only allows it if it's coming from a
local connection.
One way to do this is an SSH tunnel, which I opened like this:

```shell
$ ssh -L 32400:localhost:32400 user@domain-or-ip
```

Then I opened <http://localhost:32400/web> in my local browser and set up Plex.

## Configuring Nginx

I wanted a nice domain with HTTPS on 443 (instead of HTTP on port 32400), so I
set up [Nginx](https://nginx.com) with [Let's Encrypt](https://letsencrypt.org)
next.

The first thing I did was setting `openFirewall` to false in the Plex
configuration.
Then I allowed port 80 and 443 for HTTP and HTTPS and all the
[Plex ports](https://github.com/NixOS/nixpkgs/blob/nixos-21.11/nixos/modules/services/misc/plex.nix#L157-L160)
except for 32400 as we want to proxy the web interface through Nginx.

```nix
services.plex = {
  openFirewall = false;
  # ...
};

networking.firewall = {
  allowedTCPPorts = [ 3005 8324 32469 80 443 ];
  allowedUDPPorts = [ 1900 5353 32410 32412 32413 32414 ];
};
```

Then I configured ACME:

```nix
security.acme.acceptTerms = true;
security.acme.defaults.email = "<your email>";
```

The default provider is Let's Encrypt, you can find their terms of service here:
[Policy and Legal Repository](https://letsencrypt.org/repository/).

Now it was time to add the Nginx service.
I used recommended settings and only PFS-enabled ciphers with AES256.
As this proxies Plex requests, I forwarded some headers as well.
Here's the code:

```nix
services.nginx = {
  enable = true;

  # Recommended settings
  recommendedGzipSettings = true;
  recommendedOptimisation = true;
  recommendedProxySettings = true;
  recommendedTlsSettings = true;

  # Only allow PFS-enabled ciphers with AES256
  sslCiphers = "AES256+EECDH:AES256+EDH:!aNULL";

  virtualHosts = {
    "<your domain>" = {
      forceSSL = true;
      enableACME = true;
      extraConfig = ''
        # Some players don't reopen a socket and playback stops totally instead of resuming after an extended pause
        send_timeout 100m;
        # Plex headers
        proxy_set_header X-Plex-Client-Identifier $http_x_plex_client_identifier;
        proxy_set_header X-Plex-Device $http_x_plex_device;
        proxy_set_header X-Plex-Device-Name $http_x_plex_device_name;
        proxy_set_header X-Plex-Platform $http_x_plex_platform;
        proxy_set_header X-Plex-Platform-Version $http_x_plex_platform_version;
        proxy_set_header X-Plex-Product $http_x_plex_product;
        proxy_set_header X-Plex-Token $http_x_plex_token;
        proxy_set_header X-Plex-Version $http_x_plex_version;
        proxy_set_header X-Plex-Nocache $http_x_plex_nocache;
        proxy_set_header X-Plex-Provides $http_x_plex_provides;
        proxy_set_header X-Plex-Device-Vendor $http_x_plex_device_vendor;
        proxy_set_header X-Plex-Model $http_x_plex_model;
        # Buffering off send to the client as soon as the data is received from Plex.
        proxy_redirect off;
        proxy_buffering off;
      '';
      locations."/" = {
        proxyPass = "http://localhost:32400";
        proxyWebsockets = true;
      };
    };
  };
};
```

If you're following along, make sure to replace `<your domain>`.

To secure things even further, I set some headers for every request:

```nix
services.nginx.commonHttpConfig = ''
  # Add HSTS header with preloading to HTTPS requests.
  # Adding this header to HTTP requests is discouraged
  map $scheme $hsts_header {
      https   "max-age=31536000; includeSubdomains; preload";
  }
  add_header Strict-Transport-Security $hsts_header;
  # Enable CSP for your services.
  #add_header Content-Security-Policy "script-src 'self'; object-src 'none'; base-uri 'none';" always;
  # Minimize information leaked to other domains
  add_header 'Referrer-Policy' 'origin-when-cross-origin';
  # Disable embedding as a frame
  add_header X-Frame-Options DENY;
  # Prevent injection of code in other mime types (XSS Attacks)
  add_header X-Content-Type-Options nosniff;
  # Enable XSS protection of the browser.
  # May be unnecessary when CSP is configured properly (see above)
  add_header X-XSS-Protection "1; mode=block";
'';
```

Finally, I ran `nixos-rebuild switch` one last time to apply the configuration.
Then I opened `https://my-domain` in a browser and started creating Plex
libraries.

## How much does it cost?

The CPX11 costs €4,75/month with backups enabled, B2 costs $0.005/GB/month
storage + $0.01/GB downloaded.
Storage pricing depends heavily on the amount of media stored and the amount
of media downloaded.
I pay around €10/month for my setup.

## Wrapping up

All that's left to do now is further configure NixOS to set a hostname,
timezone, installed packages like `htop` and enabling
[Automatic Upgrades](https://nixos.org/manual/nixos/stable/index.html#sec-upgrading-automatic).

In case that's useful for you, here is the
[configuration.nix](/articles/plex-on-nixos/configuration.nix)
from when I tested this blog post.

If you discover an issue or have a question, please don't hesitate to
[let me know](mailto:hey@arne.me), I'm more than happy to help!
