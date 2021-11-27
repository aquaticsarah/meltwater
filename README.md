# Meltwater: An experimental lo-fi audio plugin

Meltwater is a [VST2][1] plugin providing a type of "lo-fi" audio degradation
which is a little different to more common methods such as sample rate / bit
depth reduction.

Meltwater works by using the [Opus audio codec][2] with a variable bitrate. This
produces a variety of interesting distortions as the codec attempts to reproduce
its input as accurately as possible in a limited amount of data. The result can
be anywhere from full transparency down to "music being played over a phone
line" quality, with many interesting points in between.

Note: This plugin is currently alpha-quality, and many features are not yet
implemented. The most important for now are:

* The plugin *will not work* unless your DAW's sample rate is set to 48kHz

* There is no GUI yet - you will need to use your DAW's built-in UI
  to set the quality parameter

Both of these issues will be resolved before v1.0

## License

Meltwater is licensed under the [GPLv3][3], a copy of which is included here as
`LICENSE.txt`. 

Each file should contain the following lines; the wording is derive from
["Copyright Notices for Open Source Projects"][4] and [the SPDX ID
guidelines][5]:

```rust
// Meltwater: [short description of file]
// Copyright [years], Sarah Ocean and the Meltwater project contributors.
// SPDX-License-Identifier: GPL-3.0-or-later
```

Note that, if you distribute a compiled version of Meltwater, this will also be
covered by the licenses of its dependencies - the Opus codec and its Rust
dependencies specified in Cargo.toml

## Contributing

Contributions are always welcome, including the addition of new methods of audio
degradation. All contributions offered for inclusion in Meltwater must be your
original work, and must be licensed under the same terms as the project as a
whole.

[1]: https://en.wikipedia.org/wiki/Virtual_Studio_Technology
[2]: https://opus-codec.org/
[3]: https://choosealicense.com/licenses/gpl-3.0/
[4]: https://ben.balter.com/2015/06/03/copyright-notices-for-websites-and-open-source-projects/
[5]: https://spdx.dev/ids/
