---
title: "DIY Music Streaming with NixOS, Jellyfin and Manet"
published: "2024-04-01"
location: "Frankfurt, Germany"
description: |
  In this post I describe how I'm hosting my own music streaming service with
  NixOS and Jellyfin on Hetzner for €6 / month.
---

In this post, I describe how I'm hosting my own music streaming service with
NixOS, Jellyfin and Manet on Hetzner for €6 / month.

If you know your way around servers, this is neither a novel nor a complicated
setup—quite the opposite; the beauty of this configuration lies in its
simplicity.

<!-- more -->

## Table of Contents

- [Set up a VM with NixOS](#set-up-a-vm-with-nixos)
- [Install and configure packages](#install-and-configure-packages)
- [Configure Jellyfin](#configure-jellyfin)
- [Apps](#apps)
- [Conclusion](#conclusion)

## Set up a VM with NixOS

The smallest (and cheapest) VM on Hetzner Cloud, CX11 with 1 VCPU and 2 GB
RAM, is more than enough[^1].

Because my music collection is still pretty small, I've decided to go with
Hetzner volumes.
If you have more than 100 GB of music, I'd recommend looking into a different
storage solution in combination with `rclone`, see
[the storage section of an earlier post](https://arne.me/blog/plex-on-nixos/#setting-up-storage).

Hetzner doesn't have a one-click NixOS image, so I chose any image and followed
the "traditional" ISO installation as described in this repo:
[nixos-install-scripts](https://github.com/nix-community/nixos-install-scripts/tree/master/hosters/hetzner-cloud).
The [NixOS docs](https://nixos.wiki/wiki/Install_NixOS_on_Hetzner_Cloud) also
recommend [nixos-infect](https://github.com/elitak/nixos-infect)—if you take
this path, I'd be genuinely interested to [hear how it worked](/contact).

Then, just to be safe, I ran `nix-channel --update` and
`nixos-rebuild switch --upgrade` on the VM to upgrade everything to the latest
version.

After finding my attached volume with `ls /dev/sd*` and mounting it with
`mount /dev/sdb /mnt/media` (replace `/dev/sdb` with the path of your volume),
I ran `nixos-generate-config` to regenerate the hardware configuration and
automatically mount it on the next boot.

Because this VM is only for music, I didn't do anything to harden the vm.
If you want more security, take a look at the excellent
[Paranoid NixOS Setup](https://xeiaso.net/blog/paranoid-nixos-2021-07-18/)
article by Xe.

## Install and configure packages

On the VM, I opened `/etc/nixos/configuration.nix` in a text editor.
Nano comes installed out of the box, but you can get vim in an ephemeral shell
with `nix-shell -p vim`.

In that file, I started by changing `networking.hostName` and `time.timeZone`.
Subsequently, I configured the installed packages:

```nix
environment.systemPackages = with pkgs; [
  jellyfin
  jellyfin-web
  jellyfin-ffmpeg
  caddy
  logrotate
];
```

### Jellyfin

I install Jellyfin itself, the official Jellyfin web client, and a modified
version of FFmpeg for
[hardware acceleration](https://jellyfin.org/docs/general/administration/hardware-acceleration/#supported-acceleration-methods).
Let's enable Jellyfin:

```nix
services.jellyfin.enable = true;
```

### Caddy

We need a reverse proxy, mostly for automatic HTTPS.  
I chose Caddy because the configuration is the shortest[^2]:

```nix
services.caddy = {
  enable = true;
  virtualHosts."jellyfin.example.com".extraConfig = ''
    reverse_proxy 127.0.0.1:8096
  '';
};
networking.firewall.allowedTCPPorts = [ 80 443 ];
```

You should replace `jellyfin.example.com` with the domain you want to use for
your Jellyfin setup. Make sure to point `A` and `AAAA` records in your domain
provider to the public IP addresses of the VM.

### Logrotate

After a few days, I was uploading more music and noticed, that my SFTP uploads
were stalled.
After hours of debugging, I found out, that this apparently was due to a very
large `/var/log/auth.log` beeing too large[^3].

`logrotate` fixes this by automatically rotating, compressing and removing this
(and other) log files.
It's enabled like this:

```nix
  services.logrotate.enable = true;
```

### Auto-upgrade

To keep maintenance low, I enable automatic system upgrades using the following
configuration:

```nix
system.autoUpgrade = {
  enable = true;
  allowReboot = true;
}
```

This checks every day at 4:40 am by default, so the chances of the VM rebooting
while I want to listen to music are very slim.

### Apply changes

Run `nixos-rebuild switch` to apply the configuration.

## Configure Jellyfin

Go to the address of your server and follow the installation wizard to set up
Jellyfin[^4].
Create a library for your music and upload some to `/mnt/media`.

## Apps

Having a good mobile app is crucial for me.
The official Jellyfin apps are excellent for configuring Jellyfin, but not
great for listening to music.

When I discovered [Manet](https://tilo.dev/manet/), an iOS app, it changed
everything.
While it looks and feels very similar to Apple Music, it works significantly
better.
André, the developer who is developing this on the side, is super responsive and
quick to fix the occasional bug.
Seriously, if you use Jellyfin and iOS, and not use this app, you're missing out.
Did I mention that it's beautiful and free?

On the web, I'm using the web client.
It's not great, but it's fine.
Hoping for Manet on macOS one day[^5].

## Conclusion

And there you have it: A self-hosted music streaming service for around €6 /
month, configured with a few lines of code and requiring little maintenance.

I've been using this for over a month now, and I couldn't be happier.

What does your setup look like? Any feedback? [Let me know!](/contact)

[^1]: I have a load average of 0.05 while listening to music. It's fine.
[^2]: Caddy adds a `Server` header by default. To remove it, add `header /* { -Server }` to your config.
[^3]:
    Very deep in Stack Overflow threads was one comment, suggesting that the
    log file was too large.
    I can't find it anymore.
    Do you have additional information?
    [I'd love to hear from you!](/contact)

[^4]: I wish this could be configured with Nix.
[^5]: No pressure, André!
