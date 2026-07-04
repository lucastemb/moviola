.PHONY: trim-silence test fmt clippy clean

# Required:
#   INPUT=/path/to/media.mp4          video or audio input, e.g. .mp4 or .mp3
# Optional:
#   OUTPUT=/path/to/output.mp4        default: <input_stem>_silence_trimmed.mp4 for video, .m4a for audio
#   SILENCE_THRESHOLD=-20             default: -20 dB
#   MIN_SILENCE_MS=250                default: 250 milliseconds
#   VIDEO_ENCODER=h264_videotoolbox   default on macOS, use libx264 to force CPU
#   VIDEO_BITRATE=6000k               default for h264_videotoolbox
trim-silence:
	@if [ -z "$(INPUT)" ]; then \
		echo "Missing required INPUT. Examples:"; \
		echo "  make trim-silence INPUT=/path/to/video.mp4"; \
		echo "  make trim-silence INPUT=/path/to/audio.mp3"; \
		exit 1; \
	fi
	MOVIOLA_TOOL="trim-silence" \
	INPUT="$(INPUT)" \
	OUTPUT="$(OUTPUT)" \
	SILENCE_THRESHOLD="$(SILENCE_THRESHOLD)" \
	MIN_SILENCE_MS="$(MIN_SILENCE_MS)" \
	VIDEO_ENCODER="$(VIDEO_ENCODER)" \
	VIDEO_BITRATE="$(VIDEO_BITRATE)" \
	cargo run

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
