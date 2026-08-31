# Moviola

Rust/ffmpeg media tools run through `make` commands. Make targets select a Moviola tool by setting `MOVIOLA_TOOL`.

## Chop video

Creates a choppy, fast-paced clip from one input video by sampling short chunks evenly across the entire source video.

Required:

- `INPUT`: path to the source video
- `MAX_DURATION_SECONDS`: exact final output duration

Optional:

- `OUTPUT`: output path. Defaults to `<input_stem>_chopped.mp4` next to the input.
- `CUT_FREQUENCY_SECONDS`: duration of each sampled chunk. Defaults to `MAX_DURATION_SECONDS / 10`.
- `VIDEO_ENCODER`: video encoder. Defaults to `libx264` for higher quality. Set to `h264_videotoolbox` for faster macOS hardware encoding.
- `VIDEO_CRF`: quality for `libx264`. Defaults to `16`; lower means higher quality/larger files.
- `VIDEO_PRESET`: compression preset for `libx264`. Defaults to `slow`.
- `VIDEO_BITRATE`: bitrate for `h264_videotoolbox`. Defaults to `30000k`.

Example:

```sh
make chop-video INPUT=/path/to/video.mp4 MAX_DURATION_SECONDS=30
```

With explicit cut frequency:

```sh
make chop-video \
  INPUT=/path/to/video.mp4 \
  MAX_DURATION_SECONDS=30 \
  CUT_FREQUENCY_SECONDS=3
```

If the input video is not long enough to produce the exact output duration, Moviola returns an error before rendering.

For example, with a 180-second input, `MAX_DURATION_SECONDS=30`, and `CUT_FREQUENCY_SECONDS=2`, Moviola creates 15 two-second chunks spaced across the source, from roughly `0:00-0:02` through `2:58-3:00`.

## Trim silence

Removes regions where the audio drops below a configurable dB threshold.

Required:

- `INPUT`: path to the source video or audio file

Optional:

- `OUTPUT`: output path. Defaults to `<input_stem>_silence_trimmed.mp4` for video or `<input_stem>_silence_trimmed.m4a` for audio.
- `SILENCE_THRESHOLD`: dB threshold. Defaults to `-20`.
- `MIN_SILENCE_MS`: minimum silence duration in milliseconds. Defaults to `250`.
- `VIDEO_ENCODER`: video encoder. Defaults to `libx264` for higher quality. Set to `h264_videotoolbox` for faster macOS hardware encoding.
- `VIDEO_CRF`: quality for `libx264`. Defaults to `16`; lower means higher quality/larger files.
- `VIDEO_PRESET`: compression preset for `libx264`. Defaults to `slow`.
- `VIDEO_BITRATE`: bitrate for `h264_videotoolbox`. Defaults to `30000k`.

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
  VIDEO_CRF=16
```

Supported video input extensions: `.mp4`, `.mov`, `.m4v`, `.mkv`, `.webm`, `.avi`, `.mpeg`, `.mpg`.

Supported audio input extensions: `.mp3`, `.wav`, `.m4a`, `.aac`, `.flac`, `.ogg`, `.opus`, `.aiff`, `.aif`.

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
