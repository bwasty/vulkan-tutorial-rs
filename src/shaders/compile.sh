#!/bin/bash
source ../../mac-env.sh
glslangValidator -V shader.vert
glslangValidator -V shader.frag
