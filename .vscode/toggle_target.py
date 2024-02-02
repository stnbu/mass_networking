#!/usr/bin/env python3

import json
import os


def toggle_target(settings_path, targets):
    # Ensure the .vscode directory exists
    os.makedirs(os.path.dirname(settings_path), exist_ok=True)

    # Initialize settings if the file doesn't exist
    if not os.path.isfile(settings_path):
        settings = {}
    else:
        # Load current settings
        with open(settings_path, "r") as file:
            settings = json.load(file)

    # Toggle the target
    current_target = settings.get("rust-analyzer.cargo.target", targets[0])
    new_target = targets[(targets.index(current_target) + 1) % len(targets)]
    settings["rust-analyzer.cargo.target"] = new_target

    # Save updated settings
    with open(settings_path, "w") as file:
        json.dump(settings, file, indent=4)

    print(f"Analysis target switched to: {new_target}")


# Assuming the script is run from the crate root
crate_root = os.getcwd()  # Gets the current working directory
settings_path = os.path.join(crate_root, ".vscode", "settings.json")
targets = ["wasm32-unknown-unknown"]

toggle_target(settings_path, targets)
