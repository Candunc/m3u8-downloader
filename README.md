# m3u8-downloader

This is now the third iteration of [twitch-dl](https://github.com/Candunc/twitch-dl), however this provides an application that is similar to the original [aria2c](https://aria2.github.io/) dependency, which downloads files with multiple threads. This fixes the primary issue with ffmpeg and most other programs I have stumbled across, where throughput can't be saturated by one chunk of the m3u8 file and thus downloading is unnecessarily long. 

Usage: m3u8-downloader url filename

Currently the first (and likely the highest quality) m3u8 file is selected. It also assumes that you pass an index of m3u8 files, of which the first is selected.
