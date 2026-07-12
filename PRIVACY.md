# Popush Privacy

> **Popush collects nothing.**
>
> There is no telemetry. No analytics. No crash reporting. No usage statistics.
> No update check. No phone-home of any kind.
>
> There is no Popush server. There is no Popush account. There is no Popush
> database. Popush does not know you exist.
>
> The only network traffic Popush generates is:
> - SSH connections to servers **you** configured
> - Git operations to remotes **you** configured
> - Optionally, and only if you explicitly enable it, read-only requests to
>   `api.github.com` using a token **you** provided
>
> Everything Popush knows about you lives in `~/.config/popush/config.toml`, on
> your machine. Open it. Read it. It is plain text and it contains no secrets.
>
> Fonts are bundled with the application, not fetched from a CDN, because a CDN
> request is a network call to a third party.
>
> This is not a policy. It is an architecture. There is nothing to collect
> because there is nowhere to send it.

There is nothing to breach, nothing to leak, nothing to subpoena, and nothing to
shut down.
