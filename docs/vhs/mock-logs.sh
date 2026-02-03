#!/bin/bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# mock-logs.sh - Simulates kubectl logs output for demo recording
# Outputs JSON log entries with realistic delays between them

sleep 0.5
echo '{"timestamp":"2026-02-01T22:14:31Z","level":"info","message":"Server listening on port 8080","mood":"sunshine"}'
sleep 0.7
echo '{"timestamp":"2026-02-01T22:14:32Z","level":"debug","message":"Loading configuration","mood":"sunshine"}'
sleep 0.6
echo '{"timestamp":"2026-02-01T22:14:33Z","level":"info","message":"Connected to database","mood":"sunshine"}'
sleep 0.8
echo '{"timestamp":"2026-02-01T22:14:35Z","level":"warn","message":"Cache miss, rebuilding index","mood":"sunshine"}'
sleep 0.5
echo '{"timestamp":"2026-02-01T22:14:36Z","level":"info","message":"Request processed successfully","mood":"sunshine"}'
sleep 0.6
