#!/usr/bin/env bash

docker run -it --name ffi-sandbox --mount src="$(pwd)/playground",target=/playground,type=bind ffi-sandbox
docker rm ffi-sandbox
