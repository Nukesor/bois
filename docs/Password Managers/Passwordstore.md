# Passwordstore integration

## Setup

If your `pass` setup uses a password protected GPG key for encryption, you need to [enable gpg-agent](https://wiki.archlinux.org/title/GnuPG#gpg-agent).
Otherwise, `pass` won't work as there's no way to provide the password to decrypt your gpg key via a CLI option.
Your key needs to be, at least temporarily, added to the gpg-agent for bois to be able to access keys.

If you're paranoid about this, just set the `default-cache-ttl` to something like a minute.

### Root

If you're using `bois` to configure your system configuration, it needs to be executed as root.
Make sure that your `root` has its own gnupg setup, as it cannot use the setup of the your normal user.
I.e. it needs its own `~/.gnugp` folder with a the key to decrypt your passwordstore.

To avoid to also having to copy and synchronize your passwordstore to root, you can set the following environment variable in your global `bois.yml` to use your normal user's.

```
# /etc/bois.yml
envs:
  PASSWORD_STORE_DIR: /home/your_user/.local/share/password-store
```

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

By calling `pass("service/kagi.com", "yaml")`, all lines after the first will be interpreted as yaml and provided.

It can then be used like this in your templates:

```
username = {{ pass("service/kagi.com", "yaml").user }}
password = {{ pass("service/kagi.com") }}
token = {{ pass("service/kagi.com", "yaml")["token"] }}
```
