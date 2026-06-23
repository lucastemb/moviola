# Moviola

Rust/ffmpeg video tools run through `make` commands. Make targets select a Moviola tool by setting `MOVIOLA_TOOL`.

## Trim silence

Removes regions where the audio drops below a configurable dB threshold.

Required:

- `INPUT`: path to the source video

Optional:

- `OUTPUT`: output path. Defaults to `<input_stem>_silence_trimmed.mp4` next to the input.
- `SILENCE_THRESHOLD`: dB threshold. Defaults to `-20`.
- `MIN_SILENCE_MS`: minimum silence duration in milliseconds. Defaults to `250`.
- `VIDEO_ENCODER`: video encoder. Defaults to `h264_videotoolbox` on macOS for faster hardware encoding, otherwise `libx264`.
- `VIDEO_BITRATE`: bitrate for `h264_videotoolbox`. Defaults to `6000k`.

Example:

```sh
make trim-silence INPUT=/path/to/video.mp4
```

With overrides:

```sh
make trim-silence \
  INPUT=/path/to/video.mov \
  OUTPUT=/path/to/output.mp4 \
  SILENCE_THRESHOLD=-25 \
  MIN_SILENCE_MS=250 \
  VIDEO_BITRATE=6000k
```

Supported input extensions: `.mp4`, `.mov`, `.m4v`, `.mkv`, `.webm`, `.avi`, `.mpeg`, `.mpg`.

## Requirements

- Rust toolchain
- `ffmpeg` and `ffprobe` on your `PATH`

## Development

```sh
make fmt
make clippy
make test
```

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
