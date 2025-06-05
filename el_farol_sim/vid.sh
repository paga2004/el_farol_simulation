#!/bin/bash

ffmpeg -framerate 10 -i output/grid_states/state_%04d.png -c:v libx264 -pix_fmt yuv420p el_farol_simulation.mp4
