# GBST patch notes for adb_client 3.2.3

Upstream crate: https://github.com/cocool97/adb_client
License: MIT. The original upstream LICENSE (copyright Corentin LIAUD) is retained in this directory.
GBST source (workspace): C:\GitHub\GBST

## Local changes

1. `src/message_devices/adb_message_transport.rs`: change the virtually unbounded USB receive timeout to 120 seconds so stalled shell calls can report an error rather than wait forever.
2. `src/message_devices/usb/usb_transport.rs`: change the host-side bulk write request chunk from one endpoint `max_packet_size` (typically a few hundred bytes) to 64 KiB, preserving the final zero-length USB packet condition when the total transfer length is divisible by the endpoint packet size. Reject zero-byte successful bulk writes to prevent an infinite loop.

## Verification

On Lenovo TB128FU / Android 13 using 317,157,219-byte `com.google.android.gms_26.32.32-android-12~17.apk`:

- Before local USB write optimization: direct USB push 53.12 sec; remote SHA-256 matched; `pm install` returned `Success`.
- After optimization: direct USB push 22.14 sec; remote SHA-256 matched; `pm install` returned `Success`.
- Native `adb.exe push -z none` baseline: 11.56 sec.
- End-to-end direct USB push+install: ~73.22 sec before, ~42.06 sec after.

The Android 13 package uninstall-then-immediate-reinstall stall is a *separate* confirmed behavior and is addressed by GBST's `crates/gbst-core/src/plan.rs` (core packages are not first uninstalled on Android 13).

GBST's application-side `crates/gbst-adb/src/lib.rs` now verifies both file length and remote SHA-256 before running `pm install`, and uses explicit completion checks and Android-side install timeouts.

## Notes

The 64 KiB unit here is **the size of a host libusb bulk write request**, not a change to Android's protocol packet format or to the sync-layer DATA limit. The operating system / USB host controller fragments a bulk transfer into USB endpoint packets.

These patches are local to the vendored copy; updating the upstream crate may override them. Re-run the same direct USB push/sha256/install integration test before upgrading to a newer library version.

Direct USB ADB installation was successful in the controlled GMS tests; a blanket inability to install APKs with adb_client was not reproduced.
