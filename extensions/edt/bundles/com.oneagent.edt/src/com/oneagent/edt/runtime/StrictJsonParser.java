package com.oneagent.edt.runtime;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

final class StrictJsonParser {
    private static final int MAX_DEPTH = 128;

    private final String source;
    private int position;

    private StrictJsonParser(String source) {
        this.source = source;
    }

    static Object parse(byte[] bytes) throws ParseFailure {
        final String decoded;
        try {
            decoded = StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(bytes))
                    .toString();
        } catch (CharacterCodingException error) {
            throw new ParseFailure();
        }

        StrictJsonParser parser = new StrictJsonParser(decoded);
        Object value = parser.parseValue(0);
        parser.skipWhitespace();
        if (parser.position != decoded.length()) {
            throw new ParseFailure();
        }
        return value;
    }

    private Object parseValue(int depth) throws ParseFailure {
        skipWhitespace();
        if (position >= source.length()) {
            throw new ParseFailure();
        }
        char current = source.charAt(position);
        return switch (current) {
            case '{' -> parseObject(depth + 1);
            case '[' -> parseArray(depth + 1);
            case '"' -> parseString();
            case 't' -> parseLiteral("true", Boolean.TRUE);
            case 'f' -> parseLiteral("false", Boolean.FALSE);
            case 'n' -> parseLiteral("null", null);
            default -> {
                if (current == '-' || isDigit(current)) {
                    yield parseNumber();
                }
                throw new ParseFailure();
            }
        };
    }

    private Map<String, Object> parseObject(int depth) throws ParseFailure {
        requireDepth(depth);
        position++;
        LinkedHashMap<String, Object> result = new LinkedHashMap<>();
        skipWhitespace();
        if (consume('}')) {
            return result;
        }
        while (true) {
            skipWhitespace();
            if (!peek('"')) {
                throw new ParseFailure();
            }
            String key = parseString();
            if (result.containsKey(key)) {
                throw new ParseFailure();
            }
            skipWhitespace();
            require(':');
            result.put(key, parseValue(depth));
            skipWhitespace();
            if (consume('}')) {
                return result;
            }
            require(',');
        }
    }

    private List<Object> parseArray(int depth) throws ParseFailure {
        requireDepth(depth);
        position++;
        ArrayList<Object> result = new ArrayList<>();
        skipWhitespace();
        if (consume(']')) {
            return result;
        }
        while (true) {
            result.add(parseValue(depth));
            skipWhitespace();
            if (consume(']')) {
                return result;
            }
            require(',');
        }
    }

    private String parseString() throws ParseFailure {
        require('"');
        StringBuilder result = new StringBuilder();
        while (position < source.length()) {
            char current = source.charAt(position++);
            if (current == '"') {
                return result.toString();
            }
            if (current == '\\') {
                appendEscape(result);
                continue;
            }
            if (current <= 0x1f || Character.isLowSurrogate(current)) {
                throw new ParseFailure();
            }
            if (Character.isHighSurrogate(current)) {
                if (position >= source.length()) {
                    throw new ParseFailure();
                }
                char low = source.charAt(position++);
                if (!Character.isLowSurrogate(low)) {
                    throw new ParseFailure();
                }
                result.append(current).append(low);
                continue;
            }
            result.append(current);
        }
        throw new ParseFailure();
    }

    private void appendEscape(StringBuilder result) throws ParseFailure {
        if (position >= source.length()) {
            throw new ParseFailure();
        }
        char escaped = source.charAt(position++);
        switch (escaped) {
            case '"', '\\', '/' -> result.append(escaped);
            case 'b' -> result.append('\b');
            case 'f' -> result.append('\f');
            case 'n' -> result.append('\n');
            case 'r' -> result.append('\r');
            case 't' -> result.append('\t');
            case 'u' -> appendUnicodeEscape(result);
            default -> throw new ParseFailure();
        }
    }

    private void appendUnicodeEscape(StringBuilder result) throws ParseFailure {
        char first = readHexCodeUnit();
        if (Character.isLowSurrogate(first)) {
            throw new ParseFailure();
        }
        if (!Character.isHighSurrogate(first)) {
            result.append(first);
            return;
        }
        if (position + 2 > source.length()
                || source.charAt(position) != '\\'
                || source.charAt(position + 1) != 'u') {
            throw new ParseFailure();
        }
        position += 2;
        char second = readHexCodeUnit();
        if (!Character.isLowSurrogate(second)) {
            throw new ParseFailure();
        }
        result.append(first).append(second);
    }

    private char readHexCodeUnit() throws ParseFailure {
        if (position + 4 > source.length()) {
            throw new ParseFailure();
        }
        int value = 0;
        for (int index = 0; index < 4; index++) {
            int digit = Character.digit(source.charAt(position++), 16);
            if (digit < 0) {
                throw new ParseFailure();
            }
            value = value * 16 + digit;
        }
        return (char) value;
    }

    private Long parseNumber() throws ParseFailure {
        int start = position;
        if (consume('-') && position >= source.length()) {
            throw new ParseFailure();
        }
        if (consume('0')) {
            if (position < source.length() && isDigit(source.charAt(position))) {
                throw new ParseFailure();
            }
        } else {
            if (position >= source.length() || !isDigitOneToNine(source.charAt(position))) {
                throw new ParseFailure();
            }
            while (position < source.length() && isDigit(source.charAt(position))) {
                position++;
            }
        }
        if (position < source.length()) {
            char suffix = source.charAt(position);
            if (suffix == '.' || suffix == 'e' || suffix == 'E') {
                throw new ParseFailure();
            }
        }
        try {
            return Long.valueOf(source.substring(start, position));
        } catch (NumberFormatException error) {
            throw new ParseFailure();
        }
    }

    private Object parseLiteral(String literal, Object value) throws ParseFailure {
        if (!source.startsWith(literal, position)) {
            throw new ParseFailure();
        }
        position += literal.length();
        return value;
    }

    private void requireDepth(int depth) throws ParseFailure {
        if (depth > MAX_DEPTH) {
            throw new ParseFailure();
        }
    }

    private void skipWhitespace() {
        while (position < source.length()) {
            char current = source.charAt(position);
            if (current != ' ' && current != '\t' && current != '\r' && current != '\n') {
                return;
            }
            position++;
        }
    }

    private boolean peek(char expected) {
        return position < source.length() && source.charAt(position) == expected;
    }

    private boolean consume(char expected) {
        if (!peek(expected)) {
            return false;
        }
        position++;
        return true;
    }

    private void require(char expected) throws ParseFailure {
        if (!consume(expected)) {
            throw new ParseFailure();
        }
    }

    private static boolean isDigit(char value) {
        return value >= '0' && value <= '9';
    }

    private static boolean isDigitOneToNine(char value) {
        return value >= '1' && value <= '9';
    }

    static final class ParseFailure extends Exception {
        private static final long serialVersionUID = 1L;

        ParseFailure() {
            super(null, null, false, false);
        }
    }
}
