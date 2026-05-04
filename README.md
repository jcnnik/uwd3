# Universal Watermark Disabler 3

A fork of [UWD2](https://github.com/machineonamission/uwd2) by [Melody](https://machineonamission.me/),
which was itself inspired by [Universal Watermark Disabler](https://github.com/pr701/universal-watermark-disabler)
by [Painter701](https://github.com/pr701).

Written in [Rust](https://www.rust-lang.org/).

## What is UWD3?

UWD3 removes the watermark in the corner of Windows Insider builds. It picks up where UWD2 left off:
on newer Insider flights (26300.8346+) Microsoft ships a new `shell32.dll` days or weeks before uploading
the matching PDB to their symbol server, causing UWD2 to fail completely. UWD3 adds fallback methods that
work with no internet connection and no cached data at all.

## How to use it

Run the exe. The watermark disappears immediately.

```
uwd3.exe
```

To make the patch survive reboots and explorer.exe restarts, run once as administrator:

```
uwd3.exe install-task
```

This registers a scheduled task that re-applies the patch automatically at every logon.
To remove it: `uwd3.exe remove-task`.

If all automatic methods fail (e.g. brand-new flight, PDB not yet uploaded, no prior run to seed the cache),
find the RVA manually and supply it:

```
uwd3.exe patch-rva <hex-rva>
```

## Disclaimers

**UWD3 does not remove the "Activate Windows" watermark.** It targets the Insider beta watermark only.

**The patch does not survive explorer.exe or system restarts** unless you run `install-task`.

UWD3 only works on x86-64 CPUs (not ARM).

UWD3 has only been tested on Windows Insider beta watermarks. It may work on similar watermarks such as
"Test Mode", but these are untested.

## How does it work?

UWD3 uses the same core technique as UWD2: it writes a single `ret` instruction to the start of
`CDesktopWatermark::s_DesktopBuildPaint` in the memory of the running `explorer.exe`, causing
the function to return immediately without drawing anything. Because this is an in-memory patch only,
it is undone as soon as explorer.exe restarts.

The improvement over UWD2 is in how that function is *located*. UWD2 has one primary method
(download the PDB from Microsoft's symbol server) and one fragile fallback (exact 32-byte prologue match).
UWD3 uses a four-stage fallback chain:

| Stage | Method | Requires |
|-------|--------|----------|
| 1 | Cached `.rva` file from a previous run | — |
| 2 | Download `shell32.pdb` from Microsoft's symbol server | Internet, PDB uploaded |
| 3 | Multi-pattern binary scan (`patterns.bin`) | A `patterns.bin` seeded by any prior PDB-authoritative run |
| 4 | Structural GDI-call scan | Nothing — works completely cold |

### Stage 3 — Multi-pattern scan

Instead of matching a single 32-byte prologue snapshot, UWD3 saves four 8-byte sub-patterns taken
from different offsets inside the function (bytes 0, 8, 30, and 60). A candidate passes only if
all four sub-patterns match at their expected relative offsets. This is far more resilient to
minor compiler-generated variations between builds.

### Stage 4 — Structural GDI-call scan

The structural scan identifies `s_DesktopBuildPaint` by what it *does* rather than what it *looks like*:
it is the only function in all of `shell32.dll` that calls `SetTextColor` with the constant `0x00FFFFFF`
(white text). The scan:

1. Parses the PE import table to find `SetTextColor`'s IAT slot
2. Scans the `.text` section for indirect `CALL [rip+X]` instructions targeting that slot
3. Keeps only call sites where `0x00FFFFFF` was loaded into the color argument register within the preceding 40 bytes
4. Uses the `.pdata` exception directory (RUNTIME_FUNCTION table) to recover the exact function start

This requires zero cached state and works immediately on any new Insider flight.
