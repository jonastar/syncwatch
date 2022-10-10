#!/bin/sh

# This gives decent quality, and burns in subtitles
ffmpeg -i "$1" -vf subtitles="$1" -c:v h264 -preset slow -crf 22 out.mp4