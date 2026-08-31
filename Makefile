.PHONY: chop-video trim-silence test fmt clippy clean

# Chop a single video by sampling CUT_FREQUENCY_SECONDS chunks evenly across the
# whole source until the final output is exactly MAX_DURATION_SECONDS.
# Required:
#   INPUT=/path/to/video.mp4
#   MAX_DURATION_SECONDS=30
# Optional:
#   OUTPUT=/path/to/output.mp4        default: <input_stem>_chopped.mp4 next to input
#   CUT_FREQUENCY_SECONDS=3           default: MAX_DURATION_SECONDS / 10
#   VIDEO_ENCODER=libx264             default: libx264 for higher quality; h264_videotoolbox for faster macOS hardware encoding
#   VIDEO_CRF=16                      default for libx264; lower means higher quality/larger files
#   VIDEO_PRESET=slow                 default for libx264; slower means better compression
#   VIDEO_BITRATE=30000k              default for h264_videotoolbox
chop-video:
	@if [ -z "$(INPUT)" ]; then \
		echo "Missing required INPUT. Example:"; \
		echo "  make chop-video INPUT=/path/to/video.mp4 MAX_DURATION_SECONDS=30"; \
		exit 1; \
	fi
	@if [ -z "$(MAX_DURATION_SECONDS)" ]; then \
		echo "Missing required MAX_DURATION_SECONDS. Example:"; \
		echo "  make chop-video INPUT=/path/to/video.mp4 MAX_DURATION_SECONDS=30"; \
		exit 1; \
	fi
	MOVIOLA_TOOL="chop-video" \
	INPUT="$(INPUT)" \
	OUTPUT="$(OUTPUT)" \
	MAX_DURATION_SECONDS="$(MAX_DURATION_SECONDS)" \
	CUT_FREQUENCY_SECONDS="$(CUT_FREQUENCY_SECONDS)" \
	VIDEO_ENCODER="$(VIDEO_ENCODER)" \
	VIDEO_CRF="$(VIDEO_CRF)" \
	VIDEO_PRESET="$(VIDEO_PRESET)" \
	VIDEO_BITRATE="$(VIDEO_BITRATE)" \
	cargo run

# Required:
#   INPUT=/path/to/media.mp4          video or audio input, e.g. .mp4 or .mp3
# Optional:
#   OUTPUT=/path/to/output.mp4        default: <input_stem>_silence_trimmed.mp4 for video, .m4a for audio
#   SILENCE_THRESHOLD=-20             default: -20 dB
#   MIN_SILENCE_MS=250                default: 250 milliseconds
#   VIDEO_ENCODER=libx264             default: libx264 for higher quality; h264_videotoolbox for faster macOS hardware encoding
#   VIDEO_CRF=16                      default for libx264; lower means higher quality/larger files
#   VIDEO_PRESET=slow                 default for libx264; slower means better compression
#   VIDEO_BITRATE=30000k              default for h264_videotoolbox
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
	VIDEO_CRF="$(VIDEO_CRF)" \
	VIDEO_PRESET="$(VIDEO_PRESET)" \
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

run-moviola:
	cargo run --bin gui
