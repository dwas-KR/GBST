# GBST

**GBST is designed to help Lenovo tablet users set up Google Basic Services more easily and conveniently.**

Instead of manually checking device information, preparing APK files, or running complex ADB commands, users can complete the entire process through a simple dashboard-based interface.

<img width="1650" height="632" alt="GBST_" src="https://github.com/user-attachments/assets/c7e9a5c4-d9cb-4f91-bea6-92204b0299da" />

GBST was developed with reference to LPMBox.

Reference project:
https://github.com/dwas-KR/LPMBox/tree/main

## Supported Languages

* English (en) / 한국어 (ko) /[Русский (ru) / 日本語 (jp)
* 繁體中文 (CN/TW) / Tiếng Việt (vi) / Ελληνικά (el) / हिन्दी (hi)
* ქართული (ka) / Nederlands (nl) / العربية (ar) / Español (es)

## How to Use GBST

1. Connect your Lenovo Android tablet to your PC using a USB data cable.
2. Enable USB debugging on the tablet.
3. Allow the USB debugging authorization prompt when it appears.
4. Launch GBST and check the connected device information on the dashboard.
5. Click **Start** in the **Google Service Install, Repair, and Update** section.
6. Wait while GBST prepares the required files and performs the guided workflow.
7. Do not disconnect the tablet from the PC until the task status changes to standby or completed.
8. Open the Log Management page to review the task results when necessary.

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

The following components have been redacted because they are part of the program’s core implementation:

* Google package removal, installation, restoration, activation, and data-clearing workflows
* Update notifications and update-control workflows
* Remote APK catalog decoding and processing workflows

Redacted code sections are replaced with the following notice:

```text
This code is part of the program's core implementation and has been commented out.
```

## Disclaimer

GBST is an independently developed project and is not affiliated with, endorsed by, or officially associated with Google, Lenovo, or any APK provider.

Use GBST only on devices that you own or are authorized to manage.

The user is solely responsible for all actions performed on the device and any resulting consequences.

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
