# docker2hosthosts

Updates the host system's /etc/hosts whenever containers go up or down.

This project was inspired by https://github.com/larsks/dnsthing.

## Usage

The easiest way to use this is to create a systemd service. See
`docker2hosthosts.service` (included in this repo) for an example of how this
may be done.

This file can be your `/etc/hosts`, or it can be a seperate file to be used
with dnsmasq to provide local DNS to your whole network.

This file will have following format:

```
<existing entries>
# ===DOCKER HOSTS===
<this will be replaced>
# ===DOCKER HOSTS===
<existing entries>
```

You don't need to add the deliminators yourself, they will be automatically
created on the first run.
