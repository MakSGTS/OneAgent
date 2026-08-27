package com.oneagent.edt;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.Optional;
import org.junit.Test;

public final class RuntimeExecutableTest {
    @Test
    public void acceptsTrimmedBareExecutableAtTheExactByteBound() {
        assertEquals(Optional.of("oneagent-mcp"), RuntimeExecutable.validate("  oneagent-mcp  "));
        String exact = "a".repeat(RuntimeExecutable.MAX_BYTES);
        assertEquals(Optional.of(exact), RuntimeExecutable.validate(exact));
        assertTrue(RuntimeExecutable.validate("a".repeat(RuntimeExecutable.MAX_BYTES + 1)).isEmpty());
    }

    @Test
    public void rejectsEmptyControlsArgumentsUrisAndShellSyntax() {
        for (String value : new String[] {
                "", "   ", "oneagent mcp", "oneagent" + '\n' + "-mcp",
                "oneagent" + '\r' + "-mcp", "oneagent" + '\0' + "mcp",
                String.valueOf((char) 34) + "oneagent-mcp" + (char) 34,
                "oneagent-mcp --flag",
                "file:///runtime", "$RUNTIME", "oneagent*", "./oneagent-mcp",
                "../oneagent-mcp"
        }) {
            assertTrue(value, RuntimeExecutable.validate(value).isEmpty());
        }
        assertTrue(RuntimeExecutable.validate(null).isEmpty());
        assertTrue(RuntimeExecutable.validate("один-агент").isEmpty());
    }

    @Test
    public void acceptsOnlyOneNormalizedAbsoluteReadableExecutableFile() {
        Path executable = Path.of(System.getProperty("java.io.tmpdir"))
                .toAbsolutePath().normalize().resolve("oneagent-mcp");
        RuntimeExecutable.FileAccess files = new RuntimeExecutable.FileAccess() {
            @Override
            public Path parse(String value) {
                return Path.of(value);
            }

            @Override
            public boolean isReadableExecutableFile(Path path) {
                return executable.equals(path);
            }
        };

        assertEquals(Optional.of(executable.toString()),
                RuntimeExecutable.validate(executable.toString(), files));
        assertTrue(RuntimeExecutable.validate(executable.resolve("..").resolve("oneagent-mcp").toString(),
                files).isEmpty());
        assertTrue(RuntimeExecutable.validate(executable.resolveSibling("missing").toString(),
                files).isEmpty());
        assertTrue(RuntimeExecutable.validate("relative/oneagent-mcp", files).isEmpty());
    }

    @Test
    public void rejectsPathParsingAndFilesystemInspectionFailures() {
        RuntimeExecutable.FileAccess invalidPath = new RuntimeExecutable.FileAccess() {
            @Override
            public Path parse(String value) {
                throw new InvalidPathException(value, "invalid");
            }

            @Override
            public boolean isReadableExecutableFile(Path path) {
                return true;
            }
        };
        assertTrue(RuntimeExecutable.validate("/runtime", invalidPath).isEmpty());

        RuntimeExecutable.FileAccess denied = new RuntimeExecutable.FileAccess() {
            @Override
            public Path parse(String value) {
                throw new SecurityException("private path");
            }

            @Override
            public boolean isReadableExecutableFile(Path path) {
                return true;
            }
        };
        assertTrue(RuntimeExecutable.validate("/runtime", denied).isEmpty());
    }
}
