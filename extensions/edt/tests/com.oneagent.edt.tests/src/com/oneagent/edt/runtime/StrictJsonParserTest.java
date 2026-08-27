package com.oneagent.edt.runtime;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public final class StrictJsonParserTest {
    @Test
    public void parsesValidClosedJsonValuesAndUnicode() throws Exception {
        Object parsed = StrictJsonParser.parse(
                "{\"text\":\"A\\uD83D\\uDE00\",\"values\":[-1,true,false,null]}"
                        .getBytes(StandardCharsets.UTF_8));

        assertTrue(parsed instanceof Map<?, ?>);
        Map<?, ?> object = (Map<?, ?>) parsed;
        assertEquals("A😀", object.get("text"));
        assertEquals(Arrays.asList(Long.valueOf(-1), Boolean.TRUE, Boolean.FALSE, null), object.get("values"));
    }

    @Test
    public void acceptsDepth128AndRejectsDepth129() throws Exception {
        String exact = "[".repeat(128) + "0" + "]".repeat(128);
        assertTrue(StrictJsonParser.parse(exact.getBytes(StandardCharsets.UTF_8)) instanceof List<?>);

        String over = "[".repeat(129) + "0" + "]".repeat(129);
        assertThrows(StrictJsonParser.ParseFailure.class,
                () -> StrictJsonParser.parse(over.getBytes(StandardCharsets.UTF_8)));
    }

    @Test
    public void rejectsDuplicateKeysTrailingValuesAndNonIntegerNumbers() {
        assertInvalid("{\"x\":1,\"x\":2}");
        assertInvalid("{} []");
        assertInvalid("01");
        assertInvalid("1.0");
        assertInvalid("1e2");
        assertInvalid("9223372036854775808");
    }

    @Test
    public void rejectsInvalidEscapesControlsAndSurrogates() {
        assertInvalid("\"\\x\"");
        assertInvalid("\"line\nbreak\"");
        assertInvalid("\"\\uD800\"");
        assertInvalid("\"\\uDC00\"");
        assertInvalid("\"\\uD800\\u0041\"");
    }

    private static void assertInvalid(String source) {
        assertThrows(StrictJsonParser.ParseFailure.class,
                () -> StrictJsonParser.parse(source.getBytes(StandardCharsets.UTF_8)));
    }
}
