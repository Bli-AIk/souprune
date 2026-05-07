package com.bliaik.prune;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

public final class ModListOrganizer {
    private ModListOrganizer() {
    }

    public static State organize(List<String> discoveredNames, List<String> preferredEnabledNames, String activeConfigName) {
        Set<String> discovered = new HashSet<>(discoveredNames);
        List<String> enabled = new ArrayList<>();

        for (String name : preferredEnabledNames) {
            if (discovered.contains(name) && !enabled.contains(name)) {
                enabled.add(name);
            }
        }

        if (activeConfigName != null && !activeConfigName.isEmpty() && discovered.contains(activeConfigName) && !enabled.contains(activeConfigName)) {
            enabled.add(activeConfigName);
        }

        if (enabled.isEmpty() && !discoveredNames.isEmpty()) {
            enabled.add(discoveredNames.get(0));
        }

        List<String> available = new ArrayList<>();
        for (String name : discoveredNames) {
            if (!enabled.contains(name)) {
                available.add(name);
            }
        }

        String active = "";
        if (activeConfigName != null && enabled.contains(activeConfigName)) {
            active = activeConfigName;
        } else if (!enabled.isEmpty()) {
            active = enabled.get(0);
        }

        return new State(enabled, available, active);
    }

    public static final class State {
        public final List<String> enabledNames;
        public final List<String> availableNames;
        public final String activeName;

        State(List<String> enabledNames, List<String> availableNames, String activeName) {
            this.enabledNames = enabledNames;
            this.availableNames = availableNames;
            this.activeName = activeName;
        }
    }
}
