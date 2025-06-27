# 1. Project Setup

1. **Initialize a Rust project with dependencies for macOS UI.**
   - Consider Tauri for cross-platform UI, or use the `cocoa`/`objc` crates for native macOS integration.
   - Set up a local SQLite database or file-based storage module.

2. **Repository standards**
   - Use `cargo fmt`, `cargo clippy`, and `cargo test` to maintain clean, linted code with unit tests (per guidelines in `AGENTS.md`).

---

# 2. Core Sampling-Based Tracking

1. **Background Scheduler**
   - Implement a task scheduler that prompts users at customizable intervals (default around 20 minutes) during working hours.
   - Allow silent mode for minimal disruption.

2. **Prompt Dialog**
   - Use a native macOS modal or Tauri dialog to ask, “What are you working on?”
   - Store responses with timestamps in the local database.

---

# 3. User Interface

1. **Status Bar Application**
   - Create a menu-bar (status bar) icon with dropdown interface as shown in `screenshots/`.
   - Provide quick access to start/stop tracking, open timesheets, and preferences.

2. **Light and Dark Modes**
   - Support macOS appearance settings for both light and dark themes.

# 4. Timesheet Management

1. **View & Edit**
   - Build an interface to review recorded activities, edit entries, merge similar items, group by project/task, or hide entries.

2. **Export Functionality**
   - Support exporting timesheets to CSV, JSON, and PDF formats.

---

# 5. Productivity & Automation

1. **Inactivity Detection**
   - Detect idle time and handle it automatically (e.g., mark as inactive or prompt the user).

2. **Scheduling Features**
   - Allow users to configure working hours or disable tracking outside those hours.

3. **Integration**
   - Implement iCloud synchronization to keep timesheets consistent across devices.
   - Provide AppleScript hooks or keyboard shortcuts for quick control.

---

# 6. Privacy & Security

1. **Data Handling**
   - Store timesheets locally and encrypt them.
   - Do not track specific applications, documents, or websites to preserve privacy.

2. **Cloud Sync**
   - Use macOS APIs for iCloud integration and ensure encrypted transfer.


# 7. Testing & Quality Assurance

1. **Unit and Integration Tests**
   - Write tests for the scheduler, database operations, and any parsing or exporting logic.

2. **Build and Release**
   - Configure a continuous integration setup to run formatting, linting, and tests.
   - Package the app for macOS distribution.
