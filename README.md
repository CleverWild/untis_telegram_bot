# Untis Telegram Bot

An automated Telegram bot for monitoring schedule changes through the WebUntis system.

## Description

This project is a Telegram bot that:

- Connects to the WebUntis system to retrieve schedule data
- Tracks schedule changes in real-time
- Automatically notifies users of any changes via Telegram
- Supports working with multiple schools and users

## Key Features

- 🔄 **Automatic monitoring** - periodic checking for schedule changes
- 📱 **Telegram notifications** - instant notifications about changes
- 🏫 **Multi-school support** - ability to connect to various educational institutions
- 🛡️ **Authentication system** - secure handling of WebUntis credentials
- 📊 **Detailed information** - comprehensive details about schedule changes
- 🌐 **Cloud deployment** - ready for cloud infrastructure deployment

## Technology Stack

- **Rust** - primary programming language
- **Teloxide** - library for working with Telegram Bot API
- **Shuttle** - deployment platform
- **WebUntis API** - integration with the schedule system
- **Tokio** - asynchronous runtime

## Project Structure

```
├── src/                    # Main application code
│   ├── main.rs            # Entry point and configuration
│   ├── work.rs            # Schedule monitoring logic
│   ├── diff_impl.rs       # Change detection
│   ├── message.rs         # Message handling
│   ├── utils.rs           # Utility functions
│   └── message_formatter/ # Notification formatting
└── untis/                 # WebUntis library
    └── src/               # API client and data types
```

## License

This project is distributed under the license specified in the LICENSE file.

## Contributing

We welcome contributions to the project! When making changes, please:

- Follow the existing code style
- Add tests for new functionality
- Update documentation when necessary

---

*This project is under active development. Functionality may change.*
