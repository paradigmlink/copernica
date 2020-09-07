# Copernicafs

In development

Copernicafs is a Filesystem in userspace (FUSE) application. It takes a mount point and mounts any Copernica Responses on the mount. Only valid Responses that have passed a security check will be mounted and exposed is a fashion that standard applications can read the files.

Note, this filesystem is read only. The user interface to this file system is slightly different:

## Create a directory

Creating a directory will send a Request out onto the network for a Response with that name. If a Response is found on the network it will populate the newly created directory in the FUSE.

## Copying a directory structure into the mount

Copying a directory structure into the mount will publish this data as a valid Copernica Response for others to be able to search for. The name of the top level directory will be the Response name. You may then communicate the name of the directory to friends who can then pull down your published Response.

## Deleting a directory in the mount

Deleting the directory in a mount will unlink the data in the database and it'll be garbage collected in due time.

## Editing a directory or its contents.

The file system is read only and thus data cannot be edited again. You will need to republish the data under a new name.

## Getting Started

Install `rustup`.

## Building

`$ rustup run nightly cargo build`

## Running copernicafs

`$ rustup run nightly cargo run copernicafs -- mount /path/to/mount`

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Paper

Please read the [paper](https://fractalide.com/fractalide.pdf).

## Authors

* **Stewart Mackenzie** - [sjmackenzie](https://github.com/sjmackenzie)

## License

This project is licensed under the MPLV2 License - see the [LICENSE](../LICENSE) file for details

