# tape

Terminal audio player.

## Installation

`tape` is obtainable from source.

The minimal requirements for building from source are:

- `git`
- `rustup` and nightly toolchain components

To prepare:

``` sh
git clone https://github.com/n977/tape
cd tape
```

To install:

``` sh
rustup install nightly
cargo +nightly build -r
```

After `cargo` has finished, you will discover two binaries in `./target/release` named `taped` and `tapectl`. Place them in your `PATH` to complete the installation. The common destination for manually compiled executables is `/usr/local/bin`.

## Usage

The installation includes two binaries called `taped` and `tapectl` for the server and client side respectively.

It is required that the server is supervised by a service manager to run correctly. If you use `systemd` as your service manager, consider writing a simple unit file:

```
[Service]
ExecStart=/usr/local/bin/taped
Restart=on-failure
```

Drop this file in `$HOME/.config/systemd/user/tape.service`.

To run the server:

``` sh
systemctl --user start tape
```

To run the client:

``` sh
tapectl <COMMAND>
```

For more information, run `tapectl -h`.

## License

See [LICENSE](LICENSE.md).
