# System management

`bois` is able to manage some of your system state:

- [Package management](./package_management/package_management.md)
  - Specify the exact set of packages that should be installed via various system package managers.
  - Automatically un-/install packages when changes in the `bois` configuration have taken place.
- [Service management](./service_management/service_management.md)
  - Specify the set of services that should be enabled.
  - Services that leave the configuration are automatically stopped and disabled.
