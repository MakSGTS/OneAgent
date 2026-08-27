package com.oneagent.edt.runtime;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.ThreadFactory;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/** A bounded, dependency-free compatibility probe for one OneAgent Runtime process. */
public final class RuntimeProbe {
    static final int MAX_FRAME_BYTES = 1_048_576;
    static final int MAX_STDERR_BYTES = 4_096;
    static final String REQUEST = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"server/discover\",\"params\":{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2026-07-28\",\"io.modelcontextprotocol/clientCapabilities\":{},\"io.modelcontextprotocol/clientInfo\":{\"name\":\"oneagent-edt\",\"version\":\"0.1.0\"}}}}\n";

    private static final Timeouts DEFAULT_TIMEOUTS = new Timeouts(5_000, 2_000, 2_000, 2_000);
    private static final TimeSource SYSTEM_TIME = System::nanoTime;

    private final ProbeProcess.Factory processFactory;
    private final TimeSource timeSource;
    private final Timeouts timeouts;

    public RuntimeProbe() {
        this(SystemProbeProcess.FACTORY, SYSTEM_TIME, DEFAULT_TIMEOUTS);
    }

    RuntimeProbe(ProbeProcess.Factory processFactory, TimeSource timeSource, Timeouts timeouts) {
        this.processFactory = Objects.requireNonNull(processFactory);
        this.timeSource = Objects.requireNonNull(timeSource);
        this.timeouts = Objects.requireNonNull(timeouts);
    }

    public ProbeResult probe(String executable, Path workingDirectory, CancellationToken cancellation)
            throws ProbeFailure {
        if (executable == null || executable.isEmpty() || workingDirectory == null
                || !workingDirectory.isAbsolute() || !Files.isDirectory(workingDirectory)
                || !Files.isReadable(workingDirectory) || cancellation == null) {
            throw failure(ProbeFailure.Category.INVALID_CONFIGURATION);
        }
        if (cancellation.isCancelled()) {
            throw failure(ProbeFailure.Category.CANCELLED);
        }

        final ProbeProcess process;
        try {
            process = processFactory.start(executable, workingDirectory);
        } catch (Exception error) {
            throw failure(ProbeFailure.Category.SPAWN_FAILED);
        }

        ReaderOwner readers = new ReaderOwner(process);
        CompletableFuture<Void> cancelled = new CompletableFuture<>();
        CancellationToken.Registration registration = cancellation.register(
                () -> cancelled.complete(null));
        ProbeFailure pendingFailure = null;
        boolean compatible = false;

        try {
            if (cancellation.isCancelled()) {
                cancelled.complete(null);
            }
            readers.start();
            writeRequest(process.stdin());

            long deadline = deadlineAfter(timeouts.responseMillis());
            byte[] frame = awaitFrame(readers, cancelled, deadline);
            if (cancellation.isCancelled() || cancelled.isDone()) {
                throw failure(ProbeFailure.Category.CANCELLED);
            }
            validateCompatibleResponse(frame);
            closeQuietly(process.stdin());

            if (!process.waitFor(timeouts.gracefulMillis(), TimeUnit.MILLISECONDS)) {
                throw failure(ProbeFailure.Category.PROCESS_FAILED);
            }
            if (cancellation.isCancelled() || cancelled.isDone()) {
                throw failure(ProbeFailure.Category.CANCELLED);
            }
            awaitReaderCompletion(readers);
            if (process.exitValue() != 0 || readers.stderrBytes() != 0 || readers.hasExtraStdout()) {
                throw failure(readers.hasExtraStdout()
                        ? ProbeFailure.Category.PROTOCOL_FAILURE
                        : ProbeFailure.Category.PROCESS_FAILED);
            }
            compatible = true;
        } catch (ProbeFailure error) {
            pendingFailure = error;
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            pendingFailure = failure(ProbeFailure.Category.CANCELLED);
        } finally {
            registration.close();
        }

        boolean cleaned = compatible
                ? readers.closeAfterExit()
                : terminate(process, readers);
        if (!cleaned) {
            throw failure(ProbeFailure.Category.SHUTDOWN_FAILED);
        }
        if (pendingFailure != null) {
            throw pendingFailure;
        }
        return ProbeResult.COMPATIBLE;
    }

    private void writeRequest(OutputStream stdin) throws ProbeFailure {
        try {
            stdin.write(REQUEST.getBytes(StandardCharsets.UTF_8));
            stdin.flush();
        } catch (IOException error) {
            throw failure(ProbeFailure.Category.PROCESS_FAILED);
        }
    }

    private byte[] awaitFrame(ReaderOwner readers, CompletableFuture<Void> cancelled, long deadline)
            throws ProbeFailure, InterruptedException {
        CompletableFuture<Object> first = CompletableFuture.anyOf(
                readers.frame(), readers.stderrOverflow(), cancelled);
        long remaining = deadline - timeSource.nanoTime();
        if (remaining <= 0) {
            throw failure(ProbeFailure.Category.TIMEOUT);
        }
        try {
            first.get(remaining, TimeUnit.NANOSECONDS);
        } catch (TimeoutException error) {
            throw failure(ProbeFailure.Category.TIMEOUT);
        } catch (ExecutionException error) {
            throw readerFailure(error.getCause());
        }
        if (cancelled.isDone()) {
            throw failure(ProbeFailure.Category.CANCELLED);
        }
        if (readers.stderrOverflow().isDone()) {
            throw failure(ProbeFailure.Category.STDERR_OVERFLOW);
        }
        try {
            return readers.frame().get();
        } catch (ExecutionException error) {
            throw readerFailure(error.getCause());
        }
    }

    private void awaitReaderCompletion(ReaderOwner readers)
            throws ProbeFailure, InterruptedException {
        try {
            readers.stdoutDone().get(timeouts.gracefulMillis(), TimeUnit.MILLISECONDS);
            readers.stderrDone().get(timeouts.gracefulMillis(), TimeUnit.MILLISECONDS);
        } catch (TimeoutException error) {
            throw failure(ProbeFailure.Category.PROCESS_FAILED);
        } catch (ExecutionException error) {
            throw readerFailure(error.getCause());
        }
    }

    private ProbeFailure readerFailure(Throwable error) {
        if (error instanceof ReaderFailure readerFailure) {
            return failure(readerFailure.category());
        }
        return failure(ProbeFailure.Category.PROCESS_FAILED);
    }

    private boolean terminate(ProbeProcess process, ReaderOwner readers) {
        closeQuietly(process.stdin());
        boolean exited = waitFor(process, timeouts.gracefulMillis());
        if (!exited) {
            process.destroy();
            exited = waitFor(process, timeouts.destroyMillis());
        }
        if (!exited) {
            process.destroyForcibly();
            exited = waitFor(process, timeouts.forceMillis());
        }
        readers.closeStreams();
        return exited && readers.stop();
    }

    private static boolean waitFor(ProbeProcess process, long timeoutMillis) {
        try {
            return !process.isAlive() || process.waitFor(timeoutMillis, TimeUnit.MILLISECONDS);
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            return false;
        }
    }

    private long deadlineAfter(long millis) {
        long nanos = TimeUnit.MILLISECONDS.toNanos(millis);
        long now = timeSource.nanoTime();
        long deadline = now + nanos;
        return deadline < now ? Long.MAX_VALUE : deadline;
    }

    private static void validateCompatibleResponse(byte[] frame) throws ProbeFailure {
        final Object parsed;
        try {
            parsed = StrictJsonParser.parse(frame);
        } catch (StrictJsonParser.ParseFailure error) {
            throw failure(ProbeFailure.Category.PROTOCOL_FAILURE);
        }

        Map<String, Object> root = requireObject(parsed, Set.of("jsonrpc", "id", "result"));
        Map<String, Object> result = requireObject(root.get("result"), Set.of(
                "resultType", "supportedVersions", "capabilities", "_meta", "ttlMs", "cacheScope"));
        List<Object> versions = requireArray(result.get("supportedVersions"), 1);
        Map<String, Object> capabilities = requireObject(result.get("capabilities"), Set.of("tools"));
        requireObject(capabilities.get("tools"), Set.of());
        Map<String, Object> metadata = requireObject(result.get("_meta"),
                Set.of("io.modelcontextprotocol/serverInfo"));
        Map<String, Object> serverInfo = requireObject(
                metadata.get("io.modelcontextprotocol/serverInfo"), Set.of("name", "version"));

        requireType(root.get("jsonrpc"), String.class);
        requireType(root.get("id"), Long.class);
        requireType(result.get("resultType"), String.class);
        requireType(versions.get(0), String.class);
        requireType(serverInfo.get("name"), String.class);
        requireType(serverInfo.get("version"), String.class);
        requireType(result.get("ttlMs"), Long.class);
        requireType(result.get("cacheScope"), String.class);

        boolean compatible = "2.0".equals(root.get("jsonrpc"))
                && Long.valueOf(1).equals(root.get("id"))
                && "complete".equals(result.get("resultType"))
                && "2026-07-28".equals(versions.get(0))
                && "oneagent".equals(serverInfo.get("name"))
                && "0.1.0".equals(serverInfo.get("version"))
                && Long.valueOf(0).equals(result.get("ttlMs"))
                && "public".equals(result.get("cacheScope"));
        if (!compatible) {
            throw failure(ProbeFailure.Category.INCOMPATIBLE_SERVER);
        }
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> requireObject(Object value, Set<String> keys)
            throws ProbeFailure {
        if (!(value instanceof Map<?, ?> map) || !map.keySet().equals(keys)) {
            throw failure(ProbeFailure.Category.PROTOCOL_FAILURE);
        }
        return (Map<String, Object>) map;
    }

    @SuppressWarnings("unchecked")
    private static List<Object> requireArray(Object value, int length) throws ProbeFailure {
        if (!(value instanceof List<?> list) || list.size() != length) {
            throw failure(ProbeFailure.Category.PROTOCOL_FAILURE);
        }
        return (List<Object>) list;
    }

    private static void requireType(Object value, Class<?> type) throws ProbeFailure {
        if (!type.isInstance(value)) {
            throw failure(ProbeFailure.Category.PROTOCOL_FAILURE);
        }
    }

    private static ProbeFailure failure(ProbeFailure.Category category) {
        return new ProbeFailure(category);
    }

    private static void closeQuietly(AutoCloseable closeable) {
        try {
            closeable.close();
        } catch (Exception ignored) {
            // Cleanup remains closed and redacted; terminal status is observed separately.
        }
    }

    interface TimeSource {
        long nanoTime();
    }

    record Timeouts(long responseMillis, long gracefulMillis, long destroyMillis, long forceMillis) {
        Timeouts {
            if (responseMillis <= 0 || gracefulMillis <= 0 || destroyMillis <= 0 || forceMillis <= 0) {
                throw new IllegalArgumentException("timeouts must be positive");
            }
        }
    }

    private static final class ReaderOwner {
        private final ProbeProcess process;
        private final ExecutorService executor;
        private final CompletableFuture<byte[]> frame = new CompletableFuture<>();
        private final CompletableFuture<Void> stderrOverflow = new CompletableFuture<>();
        private final CompletableFuture<Boolean> stdoutDone = new CompletableFuture<>();
        private final CompletableFuture<Integer> stderrDone = new CompletableFuture<>();

        ReaderOwner(ProbeProcess process) {
            this.process = process;
            ThreadFactory factory = runnable -> {
                Thread thread = new Thread(runnable, "oneagent-edt-runtime-probe-reader");
                thread.setDaemon(true);
                return thread;
            };
            executor = Executors.newFixedThreadPool(2, factory);
        }

        void start() {
            executor.execute(this::readStdout);
            executor.execute(this::readStderr);
        }

        CompletableFuture<byte[]> frame() {
            return frame;
        }

        CompletableFuture<Void> stderrOverflow() {
            return stderrOverflow;
        }

        CompletableFuture<Boolean> stdoutDone() {
            return stdoutDone;
        }

        CompletableFuture<Integer> stderrDone() {
            return stderrDone;
        }

        boolean hasExtraStdout() {
            return stdoutDone.getNow(Boolean.FALSE);
        }

        int stderrBytes() {
            return stderrDone.getNow(0);
        }

        boolean closeAfterExit() {
            closeStreams();
            return stop();
        }

        void closeStreams() {
            closeQuietly(process.stdin());
            closeQuietly(process.stdout());
            closeQuietly(process.stderr());
        }

        boolean stop() {
            executor.shutdownNow();
            try {
                return executor.awaitTermination(2_000, TimeUnit.MILLISECONDS);
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
                return false;
            }
        }

        private void readStdout() {
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            boolean framed = false;
            boolean extra = false;
            try {
                int next;
                while ((next = process.stdout().read()) != -1) {
                    if (framed) {
                        extra = true;
                        continue;
                    }
                    if (next == '\n') {
                        byte[] complete = bytes.toByteArray();
                        int length = complete.length;
                        if (length > 0 && complete[length - 1] == '\r') {
                            complete = Arrays.copyOf(complete, length - 1);
                        }
                        if (complete.length > MAX_FRAME_BYTES) {
                            throw new ReaderFailure(ProbeFailure.Category.PROTOCOL_FAILURE);
                        }
                        framed = true;
                        frame.complete(complete);
                        continue;
                    }
                    if (bytes.size() > MAX_FRAME_BYTES
                            || (bytes.size() == MAX_FRAME_BYTES && next != '\r')) {
                        throw new ReaderFailure(ProbeFailure.Category.PROTOCOL_FAILURE);
                    }
                    bytes.write(next);
                }
                if (!framed) {
                    throw new ReaderFailure(ProbeFailure.Category.PROTOCOL_FAILURE);
                }
                stdoutDone.complete(extra);
            } catch (ReaderFailure error) {
                frame.completeExceptionally(error);
                stdoutDone.completeExceptionally(error);
            } catch (IOException error) {
                ReaderFailure failure = new ReaderFailure(ProbeFailure.Category.PROCESS_FAILED);
                frame.completeExceptionally(failure);
                stdoutDone.completeExceptionally(failure);
            }
        }

        private void readStderr() {
            int total = 0;
            byte[] buffer = new byte[512];
            try {
                int count;
                while ((count = process.stderr().read(buffer)) != -1) {
                    total = Math.addExact(total, count);
                    if (total > MAX_STDERR_BYTES) {
                        ReaderFailure failure = new ReaderFailure(ProbeFailure.Category.STDERR_OVERFLOW);
                        stderrOverflow.completeExceptionally(failure);
                        stderrDone.completeExceptionally(failure);
                        return;
                    }
                }
                stderrDone.complete(total);
            } catch (ArithmeticException | IOException error) {
                ReaderFailure failure = new ReaderFailure(ProbeFailure.Category.PROCESS_FAILED);
                stderrOverflow.completeExceptionally(failure);
                stderrDone.completeExceptionally(failure);
            }
        }
    }

    private static final class ReaderFailure extends RuntimeException {
        private static final long serialVersionUID = 1L;
        private final ProbeFailure.Category category;

        ReaderFailure(ProbeFailure.Category category) {
            super(null, null, false, false);
            this.category = category;
        }

        ProbeFailure.Category category() {
            return category;
        }
    }
}
