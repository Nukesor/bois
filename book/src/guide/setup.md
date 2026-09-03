# Setup

To get started, just run `bois init` inside of an empty directory, or run `bois init <dir_name>` to let `bois` create the directory for you.

It will then create the following directory structure with `<hostname>` being the hostname of your machine.

```
 📁 traits/
 │ 📂 base/
 │ │ └ base.yml
 📂 hosts/
 │ 📂 <hostname>/
 │ │ <hostname>.yml
 │ │ └ vars.yml
 └ bois.yml
```
