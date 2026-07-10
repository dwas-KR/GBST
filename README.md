# GBST

**GBST (Google Basic Service Tool)** is a desktop utility for Lenovo Android tablets.  
It provides a guided USB ADB workflow to prepare, install, repair, and refresh Google Basic Service functionality on supported Lenovo devices.

GBST was developed with reference to LPMBox.  
Reference: https://github.com/dwas-KR/LPMBox/tree/main

## Overview

GBST is designed to make Google Basic Service setup easier for Lenovo tablet users.  
Instead of manually checking device information, preparing APK files, and following complex ADB steps, users can complete the process through a simple dashboard-based interface.

## Main Features

- Lenovo tablet information display
- Android version and ROM type display
- Google Basic Service status check
- Guided Google service install, repair, and update workflow
- APK preparation and download progress popup
- Task status indicator
- Log management page
- Settings page
- Multilingual interface
- Developer YouTube, feedback, and support links

## How to Use GBST

1. Connect a Lenovo Android tablet to the PC with a USB data cable.
2. Enable USB debugging on the tablet.
3. Allow the USB debugging permission prompt when it appears.
4. Open GBST and check the device information on the dashboard.
5. Click **Start** in the **Google Service Install/Repair/Update** section.
6. Wait while GBST prepares the required files and performs the guided workflow.
7. Do not disconnect the tablet until the task status returns to standby or completion.
8. Check the log page if you need to review the task result.

## Dashboard

The dashboard shows the connected tablet information, including:

- Model name
- Android version
- Manufacturer
- ROM type
- Google service status

It also provides quick access to:

- Support the developer
- Google service install/repair/update
- Developer YouTube

## Sidebar Menu

The sidebar provides quick navigation to the main areas of the app:

- **Dashboard**: View device status and start the main workflow.
- **Google Task Start**: Start the Google Basic Service workflow.
- **Log Management**: Review task logs.
- **Settings**: Change language, check updates, open feedback, and access developer links.

## Settings

The settings page includes:

- Language selection
- Developer YouTube link
- Support link
- Program update check
- Feedback link

The selected language is saved and reused when GBST is opened again.

## Logs

GBST records task progress during the workflow.  
Logs can help identify USB authorization problems, download issues, device detection problems, or workflow failures.

## Public Source Notice

Some implementation details are intentionally not included in this public source release.  
The following parts are redacted because they are part of the program's core implementation:

- Google package remove/install/restore/enable/clear workflow
- Update notification and update-control handling workflow
- Remote APK catalog decoding and processing workflow

Redacted sections are replaced with the following notice:

```text
This code is part of the program's core implementation and has been commented out.
```

## Disclaimer

GBST is an independent project and is not affiliated with Google, Lenovo, or any APK provider.  
Use it only on devices you own or are authorized to manage.  
The user is responsible for all actions performed on the device.

## License

This project is licensed under **CC BY-NC-SA 4.0** unless stated otherwise.  
Commercial use is not permitted without separate permission.
