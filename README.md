# AI Agent Activity Interface

A lightweight CLI interface to **turn AI agent activities on/off**, inspect status, and review activity history.

## Run

```bash
python3 agent_interface.py
```

## Commands

- `add <name>|<description>`: Add a new activity.
- `on <name>`: Turn activity ON.
- `off <name>`: Turn activity OFF.
- `list`: List all activities and their states.
- `history <name>`: View timestamped ON/OFF history.
- `quit`: Exit the interface.

## Example

```text
> add web_search|Use web tools for research
Added activity 'web_search'.

> on web_search
web_search turned ON.

> list
Current AI Agent Activities:
- web_search: ON | Use web tools for research
```
