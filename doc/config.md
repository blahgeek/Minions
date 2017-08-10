## Configuration

Minions uses configuration file to customize its behaviour.

By default, Minions would use contents in [default.toml](../config/default.toml) as configuration.
It `~/.minions/config.toml` exists, existing sections would override ones in default.toml, but non-existing sections would still taken from default.toml. See the default.toml for complete list of configuration options.

For example, if `~/.minions/config.toml` contains the following content:

```toml
[file_browser]
    [[file_browser.entries]]
    name = "Home Directory"
    path = "/home/blahgeek/"
    [[file_browser.entries]]
    name = "Root Directory"
    path = "/"
```

The whole "file_browser" section would be over-written with new values, leaves all other sections default.
