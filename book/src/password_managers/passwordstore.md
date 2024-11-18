# Passwordstore (`pass`)

The `pass()` templating function can be used to interact with `pass`.
There're however a few requirements for this to go smoothly:

1. If your key has a passphrase, you should have a working [gpg-agent](https://wiki.archlinux.org/title/GnuPG#gpg-agent) setup.
   Otherwise, `pass` won't work as there's no way to provide the password to decrypt your gpg key via a CLI option.
   Your key needs to be, at least temporarily, added to the gpg-agent for bois to be able to access keys.
1. When you're running `bois` as `root` to configure your system, you must have a working passwordstore and gpg setup for `root` as well.
   - To avoid to also having to copy and synchronize your passwordstore to root, you can set the following environment variable in your global `bois.yml` to use your normal user's.
     ```
     # /etc/bois.yml
     envs:
       PASSWORD_STORE_DIR: /home/your_user/.local/share/password-store
     ```

## How to use

To get data stored in `pass`, there exists the `pass` template function.

For normal password retrieval, it can be used like this in any file with activated templating: \
`{{ pass("service/kagi.com") }}` \

This will read the first line of the `service/kagi.com` file and return it.

On top of this, the function also supports deserialization of extra data.

### Function

```django,jinja
{{ pass(key, parse_mode) }}
```

The `pass` function accepts two parameters, the second being optional:

- `key` is the path you would specify when calling `pass` directly from the cli.
  If only the `key` is provided, the first line of the password file is returned.
- `parse_mode` (optional): Can be one of `["yaml"]` (feel free to contribute more formats). \
  If this is provided, the first line of the password file **is ignored** and the remaining content is interpreted as said data format.
  The content of that data format is simply returned from the function and can be further used.

## Examples

Consider the following `pass` file at `service/kagi.com`:

```yml
my super secret pass

user: my@email.de
```

### Simple Usage

A simple call that returns the first line of said file.

```django,jinja
{{ pass("service/kagi.com") }}
```

Would return: `my super secret pass`

### With Data Format

Interpret the passwordstore file as a dataformat and returns the data for further usage.
Note: The first line is always ignored.

```django,jinja
{{ pass("service/kagi.com")["user"] }}
```

Would return: `my@email.de`
