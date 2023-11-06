#!/bin/bash

SCRIPT_PATH=${0%/*}
CLASS_PATH=$(realpath "${SCRIPT_PATH}/build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar")

cd $SCRIPT_PATH && java -cp $CLASS_PATH DoQuery idx