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

## License
This work is licensed under the **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License (CC BY-NC-SA 4.0)**. However, components included with or used by GBST may be subject to separate licenses and distribution terms, which users are responsible for reviewing and complying with.

> This work is licensed under the  
> **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)**.

For full details, see the `LICENSE` file or visit: 
[https://creativecommons.org/licenses/by-nc-sa/4.0/](https://creativecommons.org/licenses/by-nc-sa/4.0/)

[![CC BY-NC-SA 4.0][cc-by-nc-sa-image]][cc-by-nc-sa]

[cc-by-nc-sa]: https://creativecommons.org/licenses/by-nc-sa/4.0/
[cc-by-nc-sa-image]: https://licensebuttons.net/l/by-nc-sa/4.0/88x31.png
