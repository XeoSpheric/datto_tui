# Datto TUI

Datto TUI is a unified terminal-based interface designed to streamline Managed Service Provider (MSP) workflows. By connecting multiple tools into a single, cohesive workspace, it aims to allow technicians to complete 90% of their daily tasks without switching between browser tabs.

Currently, the project is a powerful TUI (Terminal User Interface), with plans to evolve into a comprehensive web interface for co-managed customers.

## Features

### Unified Dashboard
Connects and aggregates data from key MSP tools:
- **Datto RMM**
- **Datto AV**
- **RocketCyber**
- **Sophos**

### Core Capabilities
- **Site & Device Management**: 
  - Browse Sites and Devices directly from Datto RMM.
  - Drill down into specific Device details (Variables, Security, Jobs).
- **Variable Management**:
  - View, Create, and Edit Site Variables.
  - View and Update Device UDFs (User Defined Fields).
- **Security Operations**:
  - **RocketCyber**: View active incident statistics.
  - **Sophos**: 
    - Monitor active and resolved cases.
    - View Endpoint details.
    - **Action**: Initiate scans directly from the interface.
  - **Datto AV**:
    - View Agent details and statuses.
    - Monitor Alerts.
    - **Action**: Initiate scans.

## Roadmap

The goal is to provide a single pane of glass for all major MSP operations.

### Upcoming Features
- **Backup Insights**: View backup statistics and statuses.
- **Search**: Global search functionality to quickly find Sites, Devices, or Alerts.
- **Sorting**: Column sorting for all data tables (Sites, Devices, Alerts, etc.).
- **Quality of Life**:
  - Keyboard shortcut improvements.
  - Enhanced error handling and visual feedback.
  - Codebase refactoring and cleanup.

### Long-Term Vision
- **Web Interface**: Transition from a pure TUI to a full-featured Web Application.
- **Customer Portal**: A dedicated login for co-managed customers to view their device information and security status.

## Getting Started

### Prerequisites
- Rust (latest stable)
- API Keys/Credentials for:
  - Datto RMM
  - Datto AV
  - RocketCyber
  - Sophos

### Configuration
1. Clone the repository.
2. Copy `.env.example` to `.env`.
3. Fill in your API credentials in the `.env` file.

### Running
```bash
cargo run
```
