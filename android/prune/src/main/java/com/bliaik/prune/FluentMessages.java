package com.bliaik.prune;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;

public final class FluentMessages {
    private final Map<String, String> messages;

    private FluentMessages(Map<String, String> messages) {
        this.messages = messages;
    }

    public static FluentMessages parse(String input) {
        Map<String, String> messages = new LinkedHashMap<>();
        String currentKey = null;
        StringBuilder currentValue = new StringBuilder();

        for (String line : input.split("\\r?\\n")) {
            if (line.trim().isEmpty() || line.trim().startsWith("#")) {
                continue;
            }

            int separator = line.indexOf('=');
            if (!line.startsWith(" ") && separator > 0) {
                if (currentKey != null) {
                    messages.put(currentKey, currentValue.toString().trim());
                }
                currentKey = line.substring(0, separator).trim();
                currentValue = new StringBuilder(line.substring(separator + 1).trim());
            } else if (currentKey != null) {
                currentValue.append('\n').append(line.trim());
            }
        }

        if (currentKey != null) {
            messages.put(currentKey, currentValue.toString().trim());
        }

        return new FluentMessages(messages);
    }

    public static FluentMessages fromStream(InputStream input) throws IOException {
        StringBuilder text = new StringBuilder();
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(input, StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                text.append(line).append('\n');
            }
        }
        return parse(text.toString());
    }

    public static String selectLanguage(String requested, Locale systemLocale) {
        if ("en".equals(requested) || "zh-Hans".equals(requested)) {
            return requested;
        }

        String tag = systemLocale.toLanguageTag();
        String language = systemLocale.getLanguage();
        String country = systemLocale.getCountry();
        if (tag.startsWith("zh-Hans") || ("zh".equals(language) && ("CN".equals(country) || "SG".equals(country)))) {
            return "zh-Hans";
        }
        return "en";
    }

    public String get(String key) {
        String value = messages.get(key);
        return value == null ? key : value;
    }

    public String format(String key, Object... pairs) {
        String value = get(key);
        for (int i = 0; i + 1 < pairs.length; i += 2) {
            value = value.replace("{ $" + pairs[i] + " }", String.valueOf(pairs[i + 1]));
        }
        return value;
    }
}
