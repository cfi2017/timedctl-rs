# Stream Deck Integration for timedctl-rs

Streamdeck Support is not yet implemented.

This document outlines various approaches for integrating timedctl-rs with the Elgato Stream Deck on Linux.

## Overview

The Stream Deck is a customizable control pad with programmable LCD buttons that can be used to automate tasks and streamline workflows. Integrating timedctl-rs with Stream Deck would allow for quick time tracking actions like:

- Starting/stopping activities
- Adding common reports with predefined durations
- Switching between common tasks
- Creating reports for predefined tasks
- Displaying time tracking status

## Approaches for Linux Integration

### 1. StreamDeck-UI

**Description**: A Linux-compatible GUI application for the Stream Deck written in Python.

**Integration Options**:
- Configure buttons to execute timedctl commands via shell scripts
- Create custom icons for different time tracking actions
- Set up multi-action sequences (e.g., stop current activity, then start a new one)

**Resources**:
- Repository: [https://github.com/timothycrosley/streamdeck-ui](https://github.com/timothycrosley/streamdeck-ui)
- Supports hotkeys, running applications, and executing shell scripts

**Advantages**:
- Fully GUI-based configuration
- No need to write custom plugins
- Regular updates and active community

**Limitations**:
- Limited dynamic content (status display)
- No direct integration with timedctl libraries

### 2. python-elgato-streamdeck Library

**Description**: A Python library for direct communication with Stream Deck devices.

**Integration Options**:
- Create a custom Python script that uses both the Stream Deck library and wraps timedctl commands
- Render dynamic icons showing current task and elapsed time
- Implement context-aware buttons that change based on the current tracking state

**Resources**:
- Repository: [https://github.com/abcminiuser/python-elgato-streamdeck](https://github.com/abcminiuser/python-elgato-streamdeck)
- Documentation: [https://python-elgato-streamdeck.readthedocs.io/](https://python-elgato-streamdeck.readthedocs.io/)

**Advantages**:
- Full control over button behavior and appearance
- Can implement dynamic content and state-aware buttons
- Direct device access without middleware

**Limitations**:
- Requires custom Python development
- Need to manage the interaction between Python and Rust code

### 3. Node.js Stream Deck SDK

**Description**: Stream Deck plugins can be created using JavaScript and the Elgato SDK.

**Integration Options**:
- Create a Node.js wrapper for timedctl-rs commands
- Develop a custom Stream Deck plugin with a web-based configuration interface
- Use WebSockets to maintain state between the plugin and timedctl

**Resources**:
- Official SDK: [https://developer.elgato.com/documentation/stream-deck/sdk/overview/](https://developer.elgato.com/documentation/stream-deck/sdk/overview/)
- Stream Deck SDK on npm: [https://www.npmjs.com/package/@elgato-stream-deck/node](https://www.npmjs.com/package/@elgato-stream-deck/node)

**Advantages**:
- Cross-platform compatibility
- Rich UI capabilities for configuration
- Official support from Elgato

**Limitations**:
- Requires running the Stream Deck application (may need Wine on Linux)
- More complex development process
- Potential overhead of Node.js environment

### 4. Direct Integration in timedctl-rs

**Description**: Extend timedctl-rs to communicate directly with the Stream Deck device.

**Integration Options**:
- Add Stream Deck support directly to timedctl-rs using Rust libraries
- Create a daemon mode that monitors Stream Deck input and updates button states
- Implement dynamic button rendering based on current tracking state

**Relevant Rust Libraries**:
- `streamdeck`: [https://crates.io/crates/streamdeck](https://crates.io/crates/streamdeck)
- `hidapi`: [https://crates.io/crates/hidapi](https://crates.io/crates/hidapi)

**Advantages**:
- Native integration without additional language dependencies
- Potentially lower resource usage
- Consistent codebase in Rust

**Limitations**:
- More development effort
- Less community examples compared to Python or Node.js options
- May require handling low-level HID communication

### 5. Stream Deck CLI

**Description**: Command-line tools to control Stream Deck devices on Linux.

**Integration Options**:
- Use CLI tools to update Stream Deck button states and icons
- Create shell scripts that combine timedctl commands with Stream Deck CLI commands
- Set up a monitoring service that updates Stream Deck based on timedctl status

**Resources**:
- streamdeck-cli: [https://github.com/Julusian/node-elgato-stream-deck](https://github.com/Julusian/node-elgato-stream-deck)
- go-streamdeck: [https://github.com/dim13/streamdeck](https://github.com/dim13/streamdeck)

**Advantages**:
- Simple integration through shell scripts
- No need for complex plugin architecture
- Works well with existing timedctl CLI

**Limitations**:
- Limited dynamic capabilities
- May require additional scripting for state management
- Potential performance overhead from frequent process spawning

## Implementation Considerations

### Button Layout and Actions

Potential button configurations:
- Start/stop current activity
- Quick task selection (most common tasks)
- Add predefined reports (1h, 30min, 15min)
- Show current activity and elapsed time
- Navigate between different pages (tasks, reports, settings)

### State Management

Options for maintaining state:
1. Poll timedctl regularly for status
2. Create a lightweight status API in timedctl-rs
3. Use filesystem-based state sharing (status files)
4. Implement a WebSocket server in timedctl for real-time updates

### Icons and Visual Feedback

- Create task-specific icons for common tasks
- Use dynamic text rendering to show elapsed time
- Use color coding for different states (active, idle)
- Consider accessibility with clear, high-contrast designs

### Security Considerations

- Stream Deck integration will have the same permissions as the user
- Consider token handling and authentication persistence
- Evaluate the security implications of storing credentials for non-interactive use

## Recommended Approach

For a balance of development effort and functionality, the following phased approach is recommended:

1. **Phase 1**: Start with StreamDeck-UI for basic command execution
   - Configure buttons to run timedctl commands
   - Create simple shell scripts for common actions
   - Test usability and gather feedback

2. **Phase 2**: Develop a Python bridge using python-elgato-streamdeck
   - Create a daemon that monitors activity state
   - Implement dynamic button updates
   - Add context-aware actions

3. **Phase 3** (optional): Consider native Rust integration
   - Evaluate the effort vs. benefit of direct integration
   - Potentially implement as a separate crate that interfaces with timedctl-rs

## Next Steps

To begin implementation:

1. Choose an initial approach based on development resources and requirements
2. Define the core set of Stream Deck actions to support
3. Design button layouts and icons
4. Create a prototype implementation
5. Test with real workflows and iterate

Some extra ideas for streamdeck integration:
- Show all activities of previous day on a deck page for easy resuming
- Show all activities of today for easy resuming

Focus the streamdeck stuff on context switching. Obviously adding new tasks is less ideal, maybe you have some ideas?

Still only conceptualising, no implementation.
