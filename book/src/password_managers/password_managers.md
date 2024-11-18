# Password Managers

`bois` provides a list of integrations with password managers to allow injecting sensitive data into your configuration files via templating.

For this purpose, each supported password manager exposes one or more functions, which might differ slighty based on the supported functionality of the respective manager.

For example, the passwordstore (`pass`) password manager can be used like this:

```config
# bois_config
# template: true
# bois_config

MY_SECRET_KEY={{ pass("secrets/root_key") }}

...

```

Take a look at the documentation for the individual managers for more detail on how to use them.
