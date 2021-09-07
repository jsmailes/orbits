# Orbits

Watch small circles orbit a slightly larger circle!

## Installation

### Binary

Binaries are available from the [releases page](https://github.com/jsmailes/orbits/releases).
Currently only tested on Fedora, testing on other systems welcome!

### Manual

Requires rust and cargo, installation instructions can be found [here](https://www.rust-lang.org/tools/install).

Clone this repository:
```
git clone https://github.com/jsmailes/orbits.git
cd orbits
```

Build and install using cargo:
```
cargo install --path .
```

### Crate

TBD

## Usage

```
orbits [FLAGS] [OPTIONS]

FLAGS:
    -f, --fullscreen    Run in fullscreen
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -n, --num_planets <num_planets>      Number of planets
    -l, --trail_length <trail_length>    Length of trails
```
