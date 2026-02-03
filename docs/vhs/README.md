# jlif Demo Recording

This directory contains VHS tape files for generating the demo video.

## Prerequisites

- [VHS](https://github.com/charmbracelet/vhs) installed
- `jlif` binary in PATH
- zsh shell

## Recording

Run the recording script:

```bash
./record.sh
```

This will:
1. Prepare a temporary demo environment
2. Run the VHS recording
3. Move output files to `docs/pages/assets/`
4. Clean up the temporary directory

## Files

- `demo.tape` - VHS tape definition
- `mock-logs.sh` - Script that simulates kubectl log output
- `record.sh` - Setup and recording script
