# Cup 🥤

Cup is the easiest way to check for container image updates without using up the Docker Hub pull rate limit.

Inspired by [What's up docker?](https://github.com/fmartinou/whats-up-docker) which would make my server useless for the next 6 hours when used...

## Installation

### Method 1 (recommended):

If you have [Docker](https://docker.com) installed, you can use the docker image we provide.
Go to the Usage section and replace `cup` with `docker run -t -v /var/run/docker.sock:/var/run/docker.sock ghcr.io/sergi0g/cup`

### Method 2 (from source):
> [!IMPORTANT]
> You will need to have Rust installed on your computer. Go to [https://rustup.rs/](https://rustup.rs/) and follow the instructions.

1. Clone the repo
```bash
git clone https://github.com/sergi0g/cup.git
```
2. Change your working directory
```bash
cd cup
```
3. Build Cup
```bash
cargo build -r
```
4. Move the binary into a directory in your PATH. You can try this for Linux:
```bash
mv ./target/release/cup ~/.local/bin
```

## Usage

To check for updates for all images:
```
$ cup
################################################## Done!                               Overall progress:   47/47
The following images have updates:
apitable/openresty:latest
node:latest
valkey/valkey:7.2-alpine

The following images couldn't be processed:
mcr.microsoft.com/devcontainers/go:0-1.19-bullseye
mcr.microsoft.com/devcontainers/javascript-node:1-18-bullseye
docker.dragonflydb.io/dragonflydb/dragonfly:latest
```

To check for updates to a specific image:
```
$ cup node:latest
Image node:latest has an update
```

You can also specify the path to the docker socket:
```
$ cup --socket /var/run/docker.sock node:latest
Image node:latest has an update
```

## Limitations

Currently Cup can only check for updates to images from Docker Hub. More registries will be added in the future!

Output is uncolored (mostly) and has no structure. Will be fixed...

## Troubleshooting

If you encounter any issues during installation or usage, please check the [issues](https://github.com/sergi0g/cup/issues) page or open a new issue.

## Contributing

If you think you can make Cup better, you can help out!

If you'd like to contribute to Cup, please fork the repository and submit a pull request.

Is a feature missing? [Open an issue](https://github.com/sergi0g/cup/issues/new) (if one doesn't exist already)

Please forgive my messy code.
