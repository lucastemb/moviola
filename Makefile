.PHONY: trim-silence test fmt clippy clean

# Required:
#   INPUT=/path/to/video.mp4
# Optional:
#   OUTPUT=/path/to/output.mp4        default: <input_stem>_silence_trimmed.mp4 next to input
#   SILENCE_THRESHOLD=-20             default: -20 dB
#   MIN_SILENCE_MS=250                default: 250 milliseconds
#   VIDEO_ENCODER=h264_videotoolbox    default on macOS, use libx264 to force CPU
#   VIDEO_BITRATE=6000k                default for h264_videotoolbox
trim-silence:
	@if [ -z "$(INPUT)" ]; then \
		echo "Missing required INPUT. Example:"; \
		echo "  make trim-silence INPUT=/path/to/video.mp4"; \
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
