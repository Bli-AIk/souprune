package com.bliaik.prune;

import org.junit.Test;

import java.util.Locale;

import static org.junit.Assert.assertEquals;

public class FluentMessagesTest {
    @Test
    public void parsesSimpleFluentMessages() {
        FluentMessages messages = FluentMessages.parse("app-title = Prune\nmods-title = Mods\n");

        assertEquals("Prune", messages.get("app-title"));
        assertEquals("Mods", messages.get("mods-title"));
    }

    @Test
    public void selectsSimplifiedChineseForSystemLocale() {
        assertEquals("zh-Hans", FluentMessages.selectLanguage("system", Locale.forLanguageTag("zh-CN")));
        assertEquals("zh-Hans", FluentMessages.selectLanguage("system", Locale.forLanguageTag("zh-Hans")));
    }

    @Test
    public void fallsBackToEnglishForUnsupportedSystemLocale() {
        assertEquals("en", FluentMessages.selectLanguage("system", Locale.forLanguageTag("fr-FR")));
    }

    @Test
    public void explicitLanguageOverridesSystemLocale() {
        assertEquals("en", FluentMessages.selectLanguage("en", Locale.forLanguageTag("zh-CN")));
        assertEquals("zh-Hans", FluentMessages.selectLanguage("zh-Hans", Locale.US));
    }
}
