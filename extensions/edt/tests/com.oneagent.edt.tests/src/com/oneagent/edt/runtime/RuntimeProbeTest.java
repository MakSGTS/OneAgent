package com.oneagent.edt.runtime;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.assertThrows;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.TimeUnit;
import org.junit.Test;

public final class RuntimeProbeTest {
    private static final Path WORKSPACE = Path.of(System.getProperty("java.io.tmpdir")).toAbsolutePath();
    private static final String COMPATIBLE = "{"
            + "\"jsonrpc\":\"2.0\","
            + "\"id\":1,"
            + "\"result\":{"
            + "\"resultType\":\"complete\","
            + "\"supportedVersions\":[\"2026-07-28\"],"
            + "\"capabilities\":{\"tools\":{}},"
            + "\"_meta\":{\"io.modelcontextprotocol/serverInfo\":{"
            + "\"name\":\"oneagent\",\"version\":\"0.1.0\"}},"
            + "\"ttlMs\":0,\"cacheScope\":\"public\"}}";

    @Test
    public void acceptsOnlyTheExactCompatibleResponseAndWritesTheExactRequest() throws Exception {
        FakeProcess process = FakeProcess.completed(COMPATIBLE + "\r\n", "", 0);
        CapturingFactory factory = new CapturingFactory(process);
        RuntimeProbe probe = probe(factory);

        assertSame(ProbeResult.COMPATIBLE, probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
        assertEquals(RuntimeProbe.REQUEST, process.stdinText());
        assertEquals(List.of("oneagent-mcp"), factory.executables);
        assertEquals(List.of(WORKSPACE), factory.workingDirectories);
        assertFalse(process.isAlive());
        assertTrue(process.stdinClosed);
        assertTrue(process.stdoutClosed);
        assertTrue(process.stderrClosed);
    }

    @Test
    public void acceptsReorderedMembers() throws Exception {
        String reordered = "{\"result\":{\"cacheScope\":\"public\",\"ttlMs\":0,"
                + "\"_meta\":{\"io.modelcontextprotocol/serverInfo\":{\"version\":\"0.1.0\","
                + "\"name\":\"oneagent\"}},\"capabilities\":{\"tools\":{}},"
                + "\"supportedVersions\":[\"2026-07-28\"],\"resultType\":\"complete\"},"
                + "\"id\":1,\"jsonrpc\":\"2.0\"}\n";
        RuntimeProbe probe = probe(new CapturingFactory(FakeProcess.completed(reordered, "", 0)));

        assertSame(ProbeResult.COMPATIBLE, probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
    }

    @Test
    public void rejectsMalformedDuplicateMissingUnknownAndTrailingResponseData() {
        assertFailure("{\"jsonrpc\":\"2.0\"\n", ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE.replaceFirst("\"id\":1", "\"id\":1,\"id\":1") + "\n",
                ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE.replace(",\"cacheScope\":\"public\"", "") + "\n",
                ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE.replace("\"ttlMs\":0", "\"ttlMs\":0,\"extra\":true") + "\n",
                ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE + "\n{}\n", ProbeFailure.Category.PROTOCOL_FAILURE);
    }

    @Test
    public void rejectsEveryIncompatibleIdentityValue() {
        assertFailure(COMPATIBLE.replace("\"id\":1", "\"id\":2") + "\n",
                ProbeFailure.Category.INCOMPATIBLE_SERVER);
        assertFailure(COMPATIBLE.replace("2026-07-28", "2026-01-01") + "\n",
                ProbeFailure.Category.INCOMPATIBLE_SERVER);
        assertFailure(COMPATIBLE.replace("\"oneagent\"", "\"another\"") + "\n",
                ProbeFailure.Category.INCOMPATIBLE_SERVER);
        assertFailure(COMPATIBLE.replace("\"0.1.0\"", "\"0.2.0\"") + "\n",
                ProbeFailure.Category.INCOMPATIBLE_SERVER);
        assertFailure(COMPATIBLE.replace("\"public\"", "\"private\"") + "\n",
                ProbeFailure.Category.INCOMPATIBLE_SERVER);
    }

    @Test
    public void rejectsInvalidUtf8NumbersUnicodeAndNesting() {
        assertFailure(new byte[] {(byte) 0xc3, (byte) 0x28, '\n'}, ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE.replace("\"ttlMs\":0", "\"ttlMs\":0.0") + "\n",
                ProbeFailure.Category.PROTOCOL_FAILURE);
        assertFailure(COMPATIBLE.replace("\"oneagent\"", "\"\\uD800\"") + "\n",
                ProbeFailure.Category.PROTOCOL_FAILURE);
        String nested = "[".repeat(129) + "0" + "]".repeat(129) + "\n";
        assertFailure(nested, ProbeFailure.Category.PROTOCOL_FAILURE);
    }

    @Test
    public void enforcesTheExactFrameAndStderrBounds() {
        byte[] compatibleBytes = COMPATIBLE.getBytes(StandardCharsets.UTF_8);
        byte[] exactFrame = new byte[RuntimeProbe.MAX_FRAME_BYTES + 1];
        System.arraycopy(compatibleBytes, 0, exactFrame, 0, compatibleBytes.length);
        java.util.Arrays.fill(exactFrame, compatibleBytes.length,
                RuntimeProbe.MAX_FRAME_BYTES, (byte) ' ');
        exactFrame[RuntimeProbe.MAX_FRAME_BYTES] = '\n';
        try {
            assertSame(ProbeResult.COMPATIBLE,
                    probe(new CapturingFactory(FakeProcess.completed(exactFrame, new byte[0], 0)))
                            .probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
        } catch (ProbeFailure error) {
            throw new AssertionError("the exact stdout frame bound must pass", error);
        }

        byte[] overFrame = new byte[RuntimeProbe.MAX_FRAME_BYTES + 2];
        System.arraycopy(compatibleBytes, 0, overFrame, 0, compatibleBytes.length);
        java.util.Arrays.fill(overFrame, compatibleBytes.length,
                RuntimeProbe.MAX_FRAME_BYTES + 1, (byte) ' ');
        overFrame[overFrame.length - 1] = '\n';
        assertFailure(overFrame, ProbeFailure.Category.PROTOCOL_FAILURE);

        assertFailure(COMPATIBLE + "\n", "x".repeat(RuntimeProbe.MAX_STDERR_BYTES),
                ProbeFailure.Category.PROCESS_FAILED);
        assertFailure(COMPATIBLE + "\n", "x".repeat(RuntimeProbe.MAX_STDERR_BYTES + 1),
                ProbeFailure.Category.STDERR_OVERFLOW);
    }

    @Test
    public void classifiesSpawnExitAndUnterminatedFailuresWithoutLeakingValues() {
        RuntimeProbe spawnFailure = probe((executable, directory) -> {
            throw new IOException("secret /private/path");
        });
        ProbeFailure spawned = assertThrows(ProbeFailure.class,
                () -> spawnFailure.probe("secret-executable", WORKSPACE, CancellationToken.NONE));
        assertEquals(ProbeFailure.Category.SPAWN_FAILED, spawned.category());
        assertEquals("spawn_failed", spawned.getMessage());
        assertNull(spawned.getCause());

        assertFailure(COMPATIBLE + "\n", "", 9, ProbeFailure.Category.PROCESS_FAILED);
        assertFailure(COMPATIBLE, ProbeFailure.Category.PROTOCOL_FAILURE);
    }

    @Test
    public void timeoutCancelsReadersAndEscalatesThroughDestroy() {
        FakeProcess process = FakeProcess.blocking(false, true);
        RuntimeProbe probe = new RuntimeProbe(new CapturingFactory(process),
                new SequenceTimeSource(0, TimeUnit.MILLISECONDS.toNanos(2)),
                new RuntimeProbe.Timeouts(1, 1, 1, 1));

        ProbeFailure failure = assertThrows(ProbeFailure.class,
                () -> probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));

        assertEquals(ProbeFailure.Category.TIMEOUT, failure.category());
        assertTrue(process.destroyed);
        assertFalse(process.destroyedForcibly);
        assertFalse(process.isAlive());
        assertTrue(process.stdoutClosed);
    }

    @Test
    public void cancellationBeforeAndAfterSpawnIsClosedAndSilent() {
        ManualCancellation before = new ManualCancellation();
        before.cancel();
        CapturingFactory unused = new CapturingFactory(FakeProcess.completed(COMPATIBLE + "\n", "", 0));
        ProbeFailure early = assertThrows(ProbeFailure.class,
                () -> probe(unused).probe("oneagent-mcp", WORKSPACE, before));
        assertEquals(ProbeFailure.Category.CANCELLED, early.category());
        assertTrue(unused.executables.isEmpty());

        ManualCancellation during = new ManualCancellation();
        FakeProcess process = FakeProcess.blocking(true, true);
        RuntimeProbe running = probe((executable, directory) -> {
            during.cancel();
            return process;
        });
        ProbeFailure cancelled = assertThrows(ProbeFailure.class,
                () -> running.probe("oneagent-mcp", WORKSPACE, during));
        assertEquals(ProbeFailure.Category.CANCELLED, cancelled.category());
        assertFalse(process.isAlive());
    }

    @Test
    public void shutdownFailureOverridesTheOriginalFailure() {
        FakeProcess process = FakeProcess.blocking(false, false);
        RuntimeProbe probe = new RuntimeProbe(new CapturingFactory(process),
                new SequenceTimeSource(0, TimeUnit.MILLISECONDS.toNanos(2)),
                new RuntimeProbe.Timeouts(1, 1, 1, 1));

        ProbeFailure failure = assertThrows(ProbeFailure.class,
                () -> probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));

        assertEquals(ProbeFailure.Category.SHUTDOWN_FAILED, failure.category());
        assertTrue(process.destroyed);
        assertTrue(process.destroyedForcibly);
    }

    @Test
    public void repeatedProbesUseFreshProcesses() throws Exception {
        CapturingFactory factory = new CapturingFactory(
                FakeProcess.completed(COMPATIBLE + "\n", "", 0),
                FakeProcess.completed(COMPATIBLE + "\n", "", 0));
        RuntimeProbe probe = probe(factory);

        assertSame(ProbeResult.COMPATIBLE, probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
        assertSame(ProbeResult.COMPATIBLE, probe.probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
        assertEquals(2, factory.executables.size());
    }

    private static RuntimeProbe probe(ProbeProcess.Factory factory) {
        return new RuntimeProbe(factory, System::nanoTime,
                new RuntimeProbe.Timeouts(1_000, 50, 50, 50));
    }

    private static void assertFailure(String stdout, ProbeFailure.Category category) {
        assertFailure(stdout.getBytes(StandardCharsets.UTF_8), "", 0, category);
    }

    private static void assertFailure(byte[] stdout, ProbeFailure.Category category) {
        assertFailure(stdout, "", 0, category);
    }

    private static void assertFailure(String stdout, String stderr, ProbeFailure.Category category) {
        assertFailure(stdout.getBytes(StandardCharsets.UTF_8), stderr, 0, category);
    }

    private static void assertFailure(String stdout, String stderr, int exit,
            ProbeFailure.Category category) {
        assertFailure(stdout.getBytes(StandardCharsets.UTF_8), stderr, exit, category);
    }

    private static void assertFailure(byte[] stdout, String stderr, int exit,
            ProbeFailure.Category category) {
        FakeProcess process = FakeProcess.completed(stdout, stderr.getBytes(StandardCharsets.UTF_8), exit);
        ProbeFailure failure = assertThrows(ProbeFailure.class,
                () -> probe(new CapturingFactory(process))
                        .probe("oneagent-mcp", WORKSPACE, CancellationToken.NONE));
        assertEquals(category, failure.category());
        assertEquals(category.code(), failure.getMessage());
        assertNull(failure.getCause());
        assertFalse(failure.getMessage().contains("oneagent-mcp"));
    }

    private static final class CapturingFactory implements ProbeProcess.Factory {
        private final ArrayDeque<ProbeProcess> processes = new ArrayDeque<>();
        private final List<String> executables = new ArrayList<>();
        private final List<Path> workingDirectories = new ArrayList<>();

        CapturingFactory(ProbeProcess... processes) {
            this.processes.addAll(List.of(processes));
        }

        @Override
        public ProbeProcess start(String executable, Path workingDirectory) {
            executables.add(executable);
            workingDirectories.add(workingDirectory);
            return processes.removeFirst();
        }
    }

    private static final class FakeProcess implements ProbeProcess {
        private final TrackingOutputStream stdin = new TrackingOutputStream();
        private final TrackingInputStream stdout;
        private final TrackingInputStream stderr;
        private final ArrayDeque<Boolean> waitResults = new ArrayDeque<>();
        private final int exit;
        private boolean alive = true;
        private boolean destroyed;
        private boolean destroyedForcibly;
        private boolean stdinClosed;
        private boolean stdoutClosed;
        private boolean stderrClosed;

        static FakeProcess completed(String stdout, String stderr, int exit) {
            return completed(stdout.getBytes(StandardCharsets.UTF_8), stderr.getBytes(StandardCharsets.UTF_8), exit);
        }

        static FakeProcess completed(byte[] stdout, byte[] stderr, int exit) {
            FakeProcess process = new FakeProcess(
                    new ByteArrayInputStream(stdout), new ByteArrayInputStream(stderr), exit);
            process.waitResults.add(true);
            return process;
        }

        static FakeProcess blocking(boolean gracefulExit, boolean destroyExit) {
            FakeProcess process = new FakeProcess(new BlockingInputStream(), new BlockingInputStream(), 0);
            process.waitResults.add(gracefulExit);
            process.waitResults.add(destroyExit);
            process.waitResults.add(false);
            return process;
        }

        FakeProcess(InputStream stdout, InputStream stderr, int exit) {
            this.stdout = new TrackingInputStream(stdout, () -> stdoutClosed = true);
            this.stderr = new TrackingInputStream(stderr, () -> stderrClosed = true);
            this.exit = exit;
        }

        String stdinText() {
            return stdin.toString(StandardCharsets.UTF_8);
        }

        @Override
        public OutputStream stdin() {
            return new OutputStream() {
                @Override
                public void write(int value) throws IOException {
                    stdin.write(value);
                }

                @Override
                public void write(byte[] bytes, int offset, int length) throws IOException {
                    stdin.write(bytes, offset, length);
                }

                @Override
                public void close() throws IOException {
                    stdinClosed = true;
                    stdin.close();
                }
            };
        }

        @Override
        public InputStream stdout() {
            return stdout;
        }

        @Override
        public InputStream stderr() {
            return stderr;
        }

        @Override
        public boolean waitFor(long timeout, TimeUnit unit) {
            boolean result = waitResults.isEmpty() ? !alive : waitResults.removeFirst();
            if (result) {
                alive = false;
            }
            return result;
        }

        @Override
        public int exitValue() {
            return exit;
        }

        @Override
        public boolean isAlive() {
            return alive;
        }

        @Override
        public void destroy() {
            destroyed = true;
        }

        @Override
        public void destroyForcibly() {
            destroyedForcibly = true;
        }
    }

    private static final class TrackingOutputStream extends ByteArrayOutputStream {
        @Override
        public void close() throws IOException {
            super.close();
        }
    }

    private static final class TrackingInputStream extends InputStream {
        private final InputStream delegate;
        private final Runnable onClose;

        TrackingInputStream(InputStream delegate, Runnable onClose) {
            this.delegate = delegate;
            this.onClose = onClose;
        }

        @Override
        public int read() throws IOException {
            return delegate.read();
        }

        @Override
        public int read(byte[] bytes, int offset, int length) throws IOException {
            return delegate.read(bytes, offset, length);
        }

        @Override
        public void close() throws IOException {
            onClose.run();
            delegate.close();
        }
    }

    private static final class BlockingInputStream extends InputStream {
        private boolean closed;

        @Override
        public synchronized int read() throws IOException {
            while (!closed) {
                try {
                    wait();
                } catch (InterruptedException error) {
                    Thread.currentThread().interrupt();
                    throw new IOException();
                }
            }
            return -1;
        }

        @Override
        public synchronized void close() {
            closed = true;
            notifyAll();
        }
    }

    private static final class SequenceTimeSource implements RuntimeProbe.TimeSource {
        private final ArrayDeque<Long> values = new ArrayDeque<>();

        SequenceTimeSource(long... values) {
            for (long value : values) {
                this.values.add(value);
            }
        }

        @Override
        public long nanoTime() {
            return values.size() == 1 ? values.getFirst() : values.removeFirst();
        }
    }

    private static final class ManualCancellation implements CancellationToken {
        private final CopyOnWriteArrayList<Runnable> listeners = new CopyOnWriteArrayList<>();
        private volatile boolean cancelled;

        void cancel() {
            cancelled = true;
            listeners.forEach(Runnable::run);
        }

        @Override
        public boolean isCancelled() {
            return cancelled;
        }

        @Override
        public Registration register(Runnable listener) {
            listeners.add(listener);
            if (cancelled) {
                listener.run();
            }
            return () -> listeners.remove(listener);
        }
    }
}
