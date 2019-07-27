#!/bin/bash

# convert multiple png files to a video file
# requires that libx264 and ffmpeg are installed
rm raw_frames.mp4

ffmpeg -r 24 \
 -f image2 \
 -start_number 1 \
 -i ./data/images/frame_%08d.png \
 -vcodec libx264 \
 -crf 25 \
 -pix_fmt yuv420p \
 raw_frames.mp4

