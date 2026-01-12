#!/bin/bash

MAPFLOW_VERSION=$(cat VERSION.txt)

git archive --format=tar.gz -9 --prefix=mapflow-${MAPFLOW_VERSION}/ --output=mapflow-${MAPFLOW_VERSION}.tar.gz HEAD
