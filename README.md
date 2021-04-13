# Keyswitch

A low-level key mapping program for Linux.

## Why

I wanted to map keybinds in a way that was independant of the desktop
environment or windowing system I was using. Specifically:

* Capslock + HJKL becomes arrow keys.
* Capslock on it's own becomes left contro.

If you've used [Karabiner-Elements](https://karabiner-elements.pqrs.org/) on
MacOS, you may recognize these keys as something it supports out of the box.

I tried doing this using `xmodmap`, but it didn't do exactly what I wanted.
Plus, it wouldn't be supported going forward when distros start replacing X with
Wayland.

## Installing

You can install the Rust toolchain and use `cargo build` and `cargo install`.
Or, you can use the nix derivation if you're into that sort of thing.

## Usage

The keybinds themselves are hardcoded right now, and there's no way to configure
them without editing the source code. I built this for myself, so I didn't
bother to support bindings other than the ones I want.

You'll probalmy need to run these commands using `sudo` to have sufficent
permissions to access the device files.

You can get a list of devices like this:

```sh
$ sudo keyswitch -l
Available devices:

/dev/input/event4   daskeyboard System Control
/dev/input/event3   daskeyboard Consumer Control
/dev/input/event2   daskeyboard
/dev/input/event1   Power Button
/dev/input/event0   Power Button
...
```

Find the name of the deivce for your keyboard, and then run `keyswitch`:

```sh
$ sudo keyswitch -n daskeyboard
```

This will grab the device, and create a new virtual device using `uinput`.  All
keys will be echoed via the virtual device, and any recognized bindings will
be mapped to the desired keys.

In another terminal, you can run `keyswitch -l` again and see the new virtual
device:

```sh
$ sudo keyswitch -l
Avaialble devices:

/dev/input/event21  Keyswitcher Virtual Input
...
```
