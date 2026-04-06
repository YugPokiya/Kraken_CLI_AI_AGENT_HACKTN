#!/usr/bin/env python3
"""Simple AI agent activity interface for turning activities on/off."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Dict, List


class ActivityState(str, Enum):
    OFF = "off"
    ON = "on"


@dataclass
class Activity:
    name: str
    description: str
    state: ActivityState = ActivityState.OFF
    history: List[str] = field(default_factory=list)

    def _stamp(self, message: str) -> None:
        ts = datetime.now(timezone.utc).isoformat(timespec="seconds")
        self.history.append(f"{ts} | {message}")

    def turn_on(self) -> str:
        if self.state == ActivityState.ON:
            return f"{self.name} is already ON."
        self.state = ActivityState.ON
        self._stamp("turned ON")
        return f"{self.name} turned ON."

    def turn_off(self) -> str:
        if self.state == ActivityState.OFF:
            return f"{self.name} is already OFF."
        self.state = ActivityState.OFF
        self._stamp("turned OFF")
        return f"{self.name} turned OFF."


class AIAgentInterface:
    """In-memory interface for controlling AI agent activities."""

    def __init__(self) -> None:
        self.activities: Dict[str, Activity] = {}

    def add_activity(self, name: str, description: str) -> str:
        key = name.strip().lower()
        if not key:
            return "Activity name cannot be empty."
        if key in self.activities:
            return f"Activity '{name}' already exists."
        self.activities[key] = Activity(name=name.strip(), description=description.strip())
        return f"Added activity '{name}'."

    def turn_activity(self, name: str, turn_on: bool) -> str:
        key = name.strip().lower()
        activity = self.activities.get(key)
        if not activity:
            return f"Activity '{name}' not found."
        return activity.turn_on() if turn_on else activity.turn_off()

    def list_activities(self) -> str:
        if not self.activities:
            return "No activities found."
        lines = ["Current AI Agent Activities:"]
        for activity in sorted(self.activities.values(), key=lambda x: x.name.lower()):
            lines.append(f"- {activity.name}: {activity.state.value.upper()} | {activity.description}")
        return "\n".join(lines)

    def show_history(self, name: str) -> str:
        key = name.strip().lower()
        activity = self.activities.get(key)
        if not activity:
            return f"Activity '{name}' not found."
        if not activity.history:
            return f"No history for '{activity.name}'."
        return "\n".join([f"History for {activity.name}:", *activity.history])


def run_cli() -> None:
    interface = AIAgentInterface()
    print("AI Agent Activity Interface")
    print("Type 'help' to see commands. Type 'quit' to exit.")

    while True:
        raw = input("\n> ").strip()
        if not raw:
            continue

        cmd, *args = raw.split(" ", 1)
        cmd = cmd.lower()
        arg = args[0] if args else ""

        if cmd in {"quit", "exit"}:
            print("Goodbye.")
            break

        if cmd == "help":
            print(
                "Commands:\n"
                "  add <name>|<description>    Add an activity\n"
                "  on <name>                   Turn an activity ON\n"
                "  off <name>                  Turn an activity OFF\n"
                "  list                        List all activities\n"
                "  history <name>              Show activity history\n"
                "  quit                        Exit interface"
            )
            continue

        if cmd == "add":
            if "|" not in arg:
                print("Usage: add <name>|<description>")
                continue
            name, description = [part.strip() for part in arg.split("|", 1)]
            print(interface.add_activity(name, description))
            continue

        if cmd in {"on", "off"}:
            if not arg:
                print(f"Usage: {cmd} <name>")
                continue
            print(interface.turn_activity(arg, turn_on=(cmd == "on")))
            continue

        if cmd == "list":
            print(interface.list_activities())
            continue

        if cmd == "history":
            if not arg:
                print("Usage: history <name>")
                continue
            print(interface.show_history(arg))
            continue

        print("Unknown command. Type 'help' for available commands.")


if __name__ == "__main__":
    run_cli()
