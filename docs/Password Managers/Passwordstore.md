# Passwordstore integration

## Setup

If your `pass` setup uses a password protected GPG key for encryption, you need to [enable gpg-agent](https://wiki.archlinux.org/title/GnuPG#gpg-agent).
Otherwise, `pass` won't work as there's no way to provide the password to decrypt your gpg key via a CLI option.
Your key needs to be, at least temporarily, added to the gpg-agent for bois to be able to access keys.

If you're paranoid about this, just set the `default-cache-ttl` to something like a minute.

## How to use

To get data stored in `pass`, there exists the `pass` template function.

For normal password retrieval, it can be used like this in any file with activated templating: `{{ pass("service/kagi.com") }}`.
This will read the first line of the `service/kagi.com` file and return it.

On top of this, `bois` supports deserialization of extra data.
Take the following `pass` content:

```
my_secret_password
user: some@user.com
token: my_secret_token
```

By calling `pass("service/kagi.com", parse_mode=yaml)`, all lines after the first will be interpreted as yaml and provided.

It can then be used like this in your templates:

```
username = {{ pass("service/kagi.com", parse_mode=yaml).user }}
password = {{ pass("service/kagi.com") }}
token = {{ pass("service/kagi.com", parse_mode=yaml)["token"] }}
```
