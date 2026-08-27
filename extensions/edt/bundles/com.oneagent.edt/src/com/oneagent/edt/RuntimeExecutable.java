package com.oneagent.edt;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.Optional;
import java.util.regex.Pattern;

final class RuntimeExecutable {
    static final int MAX_BYTES = 4_096;
    private static final Pattern BARE_EXECUTABLE = Pattern.compile("[A-Za-z0-9._-]+");
    private static final FileAccess SYSTEM_FILES = new FileAccess() {
        @Override
        public Path parse(String value) {
            return Path.of(value);
        }

        @Override
        public boolean isReadableExecutableFile(Path path) {
            return Files.isRegularFile(path) && Files.isReadable(path) && Files.isExecutable(path);
        }
    };

    private RuntimeExecutable() {
    }

    static Optional<String> validate(String configured) {
        return validate(configured, SYSTEM_FILES);
    }

    static Optional<String> validate(String configured, FileAccess files) {
        if (configured == null || files == null
                || configured.indexOf('\0') >= 0
                || configured.indexOf('\r') >= 0
                || configured.indexOf('\n') >= 0) {
            return Optional.empty();
        }

        String value = configured.strip();
        if (value.length() > MAX_BYTES) {
            return Optional.empty();
        }
        int bytes = value.getBytes(StandardCharsets.UTF_8).length;
        if (bytes < 1 || bytes > MAX_BYTES) {
            return Optional.empty();
        }
        if (!containsSeparator(value)) {
            return BARE_EXECUTABLE.matcher(value).matches()
                    ? Optional.of(value)
                    : Optional.empty();
        }

        try {
            Path path = files.parse(value);
            if (!path.isAbsolute() || !path.equals(path.normalize())
                    || !files.isReadableExecutableFile(path)) {
                return Optional.empty();
            }
            return Optional.of(path.toString());
        } catch (InvalidPathException | SecurityException error) {
            return Optional.empty();
        }
    }

    private static boolean containsSeparator(String value) {
        return value.indexOf('/') >= 0 || value.indexOf('\\') >= 0;
    }

    interface FileAccess {
        Path parse(String value);

        boolean isReadableExecutableFile(Path path);
    }
}
