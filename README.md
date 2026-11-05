# GBST (Google Basic Service Tool)

**GBST is designed to help Lenovo tablet users set up Google Basic Services more easily and conveniently.**

Instead of manually checking device information, preparing APK files, or running complex ADB commands, users can complete the entire process through a simple dashboard-based interface.

<img width="1650" height="632" alt="GBST_" src="https://github.com/user-attachments/assets/c7e9a5c4-d9cb-4f91-bea6-92204b0299da" />

GBST was developed with reference to LPMBox.

Reference project: https://github.com/dwas-KR/LPMBox/tree/main

## Supported Languages

* English (en) / 한국어 (ko) / Русский (ru) / 日本語 (jp)
* 繁體中文 (CN/TW) / Tiếng Việt (vi) / Ελληνικά (el) / हिन्दी (hi)
* ქართული (ka) / Nederlands (nl) / العربية (ar) / Español (es)

## APK Source and File Integrity

The APK files used by GBST are obtained through the following process:

1. The original APK files are downloaded from **APKMirror**.
2. The unmodified APK files are published as assets in the public **GBST-APK GitHub Release**.
3. GBST reads a plain-text `GBST_apk.txt` catalog and downloads the matching GitHub Release assets for the connected Android version. No Base64 decoding is used for the APK catalog. The same public catalog is also bundled with the source as a fallback.
4. Downloaded APK files are cached under `%LOCALAPPDATA%\dwas_kr\GBST\apk\Android <version>` and reused after package validation.

The APK files provided through GBST are stored and distributed **without modification, patching, malware injection, or other alterations**.

GBST does not rebuild, re-sign, or modify the original APK packages downloaded from APKMirror.

## How to Use GBST

1. Connect your Lenovo Android tablet to your PC using a USB data cable.
2. Enable USB debugging on the tablet:

   * Open **Settings**.
   * Select **About Tablet** at the bottom of the left-hand menu.
   * Scroll down on the right and quickly tap **Software Version** 10 times.
   * Developer mode is enabled when a developer notification appears.
   * Select **General Settings** from the left-hand menu.
   * Scroll to the bottom of the right-hand menu and open **Developer Options**.
   * Enable **USB Debugging**.
3. When the USB debugging authorization prompt appears on the tablet, tap **Allow**.
4. Wait for GBST to finish downloading the required APK files.
5. After the download is complete, click **Start** in the **Google Services Install/Repair/Update** section.
6. Do not disconnect the tablet from the PC until the task is complete. Refer to the logs for detailed progress and results.

## Sidebar Menu

The sidebar provides quick access to the main areas of the program:

* **Dashboard:** View the connected device status and start the main workflow.
* **Start Google Task:** Start the Google Basic Services installation, repair, and update workflow.
* **Log Management:** Review task progress and result logs.
* **Settings:** Change the language, check for program updates, submit feedback, and access developer links.

## Logs

GBST records the progress and results of each step while a task is running.

The logs can help identify issues such as:

* USB debugging authorization problems
* File download issues
* Device connection or detection problems
* Errors that occur during the workflow


## GBST v2.0.0 behavior and ADB diagnostics

- LTBox v3.3.5-compatible initial/minimum window geometry: **820 x 720 logical pixels**; the window remains resizable.
- If an external ADB server occupies the USB interface, the Dashboard displays a warning and a **Stop ADB Server** action. When idle, it sends the host:kill protocol command to 127.0.0.1:5037; after completion the app probes USB again. It does not terminate unrelated Fastboot processes or execute a system-wide taskkill.
- Google Services installation and repair still follows the pre-existing **four-stage sequence** and its existing ADB command order, including the Android 13 special handling. This update does not add a running-task cancellation button.
- Android 18 recognition is intentionally retained, but its APK catalog is not implemented and the install process cannot complete without the required APK entries.

### Tablet folder cleanup (intentional destructive behavior)

Before and after a Google Services operation, GBST attempts to delete all four folders below, including any content within them:

    /storage/self/primary/dwas_kr
    /storage/self/primary/GBST
    /data/local/tmp/dwas_kr
    /data/local/tmp/GBST

This deletion is intentionally broader than GBST's own staging subdirectory and can erase unrelated data stored in these folder names. Back up any needed data before use. Cleanup checks Lenovo device identity, refuses redirected folder roots, retries temporary connection failures, and verifies that each folder no longer exists.

### Diagnostics and log visibility

The activity log displays command failures and warnings, including failures from steps that use a continue-on-error policy. Original source log events are preserved for translation; ADB server termination results are shown in the selected language.

## License and Copyright

**GBST v2.0.0 — GNU General Public License version 3 (GPLv3).**

In the application interface this license is displayed simply as **GPLv3**.
The standardized identifier GPL-3.0-only must remain in Cargo manifests
and source headers to accurately identify the legal terms. The LTBox
reference label GPL-3.0-or-later refers to that separate project and
does not change the GBST license.


The complete, unmodified GPLv3 text is provided in LICENSE. The
original GBST program source code, Rust workspaces, build scripts, and
maintainer-authored localization and configuration code are made
available under **GNU GPLv3**, insofar as their copyright holders have
authority to provide that license. This is a strong copyleft license:
anyone distributing the GPL-covered combined program (including modified
versions) must comply with GPLv3 and provide its Corresponding Source
under the GPL. This includes source files, required build scripts and
the preferred form for modifying covered work, subject to the license's
defined exceptions. Independent unrelated works are not automatically
covered solely because they are stored alongside this project.

GPLv3 **permits commercial use, paid resale, modification, and redistribution**.
The official GBST build may be offered for free, but other people may also
sell GPL-compliant copies. No resale or commercial-use prohibition is
imposed on GPL-covered code. This license does not require anyone to
publicly release changes that are used privately and are not conveyed.

Copyright (C) 2026 DWAS (돠스), and the other actual copyright holders
of their respective contributions. The copyright owners retain their
copyright. Granting GPLv3 permissions is **not** a transfer or waiver
of copyright ownership.

### DWAS and GBST names, trademarks, logos, and images

The names **DWAS**, **돠스**, and **GBST**, and associated distinctive logos,
brand symbols, and official identity are subject to the rights of their
actual owners, including copyright (for sufficiently original artwork),
and applicable trademark and unfair-competition rights. GPLv3 section 7
allows licensors to decline to grant trademark-law rights. A GPL license
to code does **not** authorize anyone to falsely present modified builds
as official DWAS releases or claim endorsement by DWAS. Honest
attribution and legally permitted reference to the project are allowed.

**Copyright and trademark rights are different.** Any original DWAS
artwork that is actually distributed as GPL-covered program material
may be copied, modified and redistributed to the extent required by
the GPL copyright license; DWAS still remains the copyright owner.
This does **not** automatically grant trademark rights for an unrelated
product's branding or for falsely suggesting official affiliation.
Standalone DWAS branding and independent promotional images, when not
covered by the GPL program license, retain their separately specified
copyright permissions. A logo is not automatically covered by GPL merely
because its filename is visible in a public repository.

The program includes some device-model pictures, third-party marks,
logos and icons. Their actual owners' permissions must be verified before
public redistribution. No GPL license is claimed over Google APKs,
Lenovo software/firmware, upstream artwork or third-party material that
DWAS does not have authority to license. Public access to a file does
not by itself establish copyright ownership or a right to relicense it.

### Complete source and materials publication

To release GBST as intended, publish the **entire Corresponding Source**
for each distributed executable: all first-party Rust crates, the
needed patched local USB ADB library source, Cargo manifests and
lockfile, build scripts, language JSON catalogs, APK catalogs and hashes,
application assets whose redistribution is authorized, and any
preferred editable source forms needed to modify the covered work.
Check image provenance and include original design source files where
they exist and are required. The repository must actually be committed
and published; selecting a license locally does not upload files.

Third-party library licenses remain in force, including the original
**MIT** license and copyright notices for vendor/adb_client; GPLv3
does not convert that library's upstream copyright license into GPL.
External dependencies must be handled under their own licenses and
applicable GPL compatibility requirements.

This is a new license for the current GBST source to the extent its
copyright holders can authorize it; rights already granted for
previously distributed copies under CC BY-NC-SA 4.0 or MPL 2.0
are not retroactively revoked. Contributions for which DWAS does not
own the copyrights require the original owners' licensing permission.
