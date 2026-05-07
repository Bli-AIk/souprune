package com.bliaik.prune;

public final class ProjectConfigFile {
    private ProjectConfigFile() {
    }

    public static String render(String modName, String language, int resolutionScale) {
        return "[project]\n"
                + "mod_name = \"" + escapeTomlString(modName) + "\"\n"
                + "language = \"" + escapeTomlString(language) + "\"\n\n"
                + "[window]\n"
                + "resolution_scale = " + resolutionScale + "\n";
    }

    private static String escapeTomlString(String value) {
        return value.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
