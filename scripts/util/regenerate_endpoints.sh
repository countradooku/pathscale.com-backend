#! /bin/bash

endpoint-gen --config-dir config/
cp generated/model.rs src/codegen/model.rs
