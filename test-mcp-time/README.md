# MCP Time Server

This is a simple MCP server that provides time and timezone conversion capabilities.

## Features

- Get current time in any timezone
- Convert time between different timezones

## Usage

Run the server:

```bash
uvx mcp-server-time
```

Or with a specific local timezone:

```bash
uvx mcp-server-time --local-timezone America/Chicago
```

## Tools

### get_current_time

Get the current time in a specific timezone:

```json
{
  "name": "get_current_time",
  "arguments": {
    "timezone": "Europe/Warsaw"
  }
}
```

### convert_time

Convert time between different timezones:

```json
{
  "name": "convert_time",
  "arguments": {
    "source_timezone": "America/New_York",
    "time": "16:30",
    "target_timezone": "Asia/Tokyo"
  }
}
```