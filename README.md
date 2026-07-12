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
2. The downloaded files are stored on the developer's personal drive for reliable distribution.
3. GBST downloads the required APK files from that drive during the installation, repair, or update workflow.

The APK files provided through GBST are stored and distributed **without modification, patching, malware injection, or other alterations**.

GBST does not rebuild, re-sign, or modify the original APK packages downloaded from APKMirror.

Users who would like to independently verify the files may compare the GBST-provided APKs with the corresponding APKMirror downloads by checking:

* Exact file size in bytes
* Package name and version
* APK certificate or signature
* Cryptographic file hashes, such as SHA-256

The download catalog used by GBST is available here:

https://github.com/dwas-KR/GBST/blob/Download/GBST_apk.txt

> APKMirror is the original source of the APK files. The developer's personal drive is used only as a distribution location for the unchanged files required by GBST.

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

## Dashboard

The dashboard displays the following information about the connected tablet:

* Model name
* Android version
* Manufacturer
* ROM type
* Google service status

It also provides quick access to the following features:

* Support the developer
* Google service installation, repair, and update
* Developer YouTube channel

## Sidebar Menu

The sidebar provides quick access to the main areas of the program:

* **Dashboard:** View the connected device status and start the main workflow.
* **Start Google Task:** Start the Google Basic Services installation, repair, and update workflow.
* **Log Management:** Review task progress and result logs.
* **Settings:** Change the language, check for program updates, submit feedback, and access developer links.

## Settings

The Settings page includes the following options:

* Language selection
* Developer YouTube link
* Support link
* Program update check
* Feedback link

The selected language is saved automatically and will remain active the next time GBST is launched.

## Logs

GBST records the progress and results of each step while a task is running.

The logs can help identify issues such as:

* USB debugging authorization problems
* File download issues
* Device connection or detection problems
* Errors that occur during the workflow

## Public Source Notice

Some implementation details are intentionally excluded from the public source release.

The following components have been redacted because they are part of the program's core implementation:

* Google package removal, installation, restoration, activation, and data-clearing workflows
* Update notifications and update-control workflows
* Remote APK catalog decoding and processing workflows

Redacted code sections are replaced with the following notice:

```text
This code is part of the program's core implementation and has been commented out.
```

The exclusion of these implementation details does not change the APK source or file-integrity policy described above. The APK files distributed through GBST are unchanged copies of the corresponding files obtained from APKMirror.

## Disclaimer

GBST is an independently developed project and is not affiliated with, endorsed by, or officially associated with Google, Lenovo, APKMirror, or any other APK provider.

Use GBST only on devices that you own or are authorized to manage.

The user is solely responsible for all actions performed on the device and any resulting consequences.

Although the developer states that the distributed APK files are not modified, users are encouraged to independently verify file hashes, signatures, versions, and file sizes before installation.

## License

> Components included with or used by GBST may be subject to their own licenses and distribution terms. Please review and comply with those terms separately.
>
> This work is licensed under the
> **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License (CC BY-NC-SA 4.0)**.
>
> For full details, see the `LICENSE` file or visit:
> https://creativecommons.org/licenses/by-nc-sa/4.0/
>
> [![CC BY-NC-SA 4.0][cc-by-nc-sa-image]][cc-by-nc-sa]
