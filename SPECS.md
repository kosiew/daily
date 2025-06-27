# macOS Time-Tracking App in Rust 

## Objective

Develop a native macOS time-tracking app using **Rust**, inspired by the functionality and philosophy of **Daily**. The app should utilize a **sampling-based approach** to time tracking by periodically prompting users (e.g., every 20 minutes) to input what task they are working on—eliminating the need for manual timers and helping maintain focus and accountability.

---

## Key Features

### 🕒 Sampling-Based Time Tracking
- Periodic prompts asking users what they're working on.
- Customizable intervals and working hours.
- Silent mode to apply activity without visual disturbance.

### 💻 User Interface
- Native macOS interface with **light & dark mode** support.
- Intuitive, minimal, and user-friendly design.
- **Status bar app** that sits in the macOS menu bar for quick access (as shown in screenshots).
- Dropdown interface accessible from the status bar icon.

### 📋 Timesheet Management
- View, edit, rename, merge, group, and hide activities.
- Generate accurate timesheets based on user responses.
- Export timesheets in **CSV, JSON, PDF** formats.

### ⚙️ Productivity & Automation
- Inactivity detection and automatic tracking.
- Tracking scheduler to define when prompts occur.
- **iCloud support** for syncing data across Apple devices.
- **AppleScript** automation and **keyboard shortcuts** for quick control.

### 🔐 Privacy & Performance
- No tracking of applications, documents, or websites.
- Data securely stored and encrypted locally and in the cloud.

---

## Technology Stack

- **Rust** with bindings to native macOS frameworks (`cocoa`, `objc`) or use of **Tauri** for cross-platform UI.
- **SQLite** or local file-based storage.
- **iCloud integration** via native macOS APIs for seamless data synchronization.

---

## 📸 Screenshots

Reference screenshots and mockups are available in the `screenshots/` folder to illustrate the desired user interface and functionality.
 
