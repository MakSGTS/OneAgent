package com.oneagent.edt.runtime;

import java.io.InputStream;
import java.io.OutputStream;
import java.util.concurrent.TimeUnit;

final class SystemProbeProcess implements ProbeProcess {
    static final Factory FACTORY = (executable, workingDirectory) -> {
        ProcessBuilder builder = new ProcessBuilder(executable);
        builder.directory(workingDirectory.toFile());
        builder.redirectErrorStream(false);
        return new SystemProbeProcess(builder.start());
    };

    private final Process process;

    private SystemProbeProcess(Process process) {
        this.process = process;
    }

    @Override
    public OutputStream stdin() {
        return process.getOutputStream();
    }

    @Override
    public InputStream stdout() {
        return process.getInputStream();
    }

    @Override
    public InputStream stderr() {
        return process.getErrorStream();
    }

    @Override
    public boolean waitFor(long timeout, TimeUnit unit) throws InterruptedException {
        return process.waitFor(timeout, unit);
    }

    @Override
    public int exitValue() {
        return process.exitValue();
    }

    @Override
    public boolean isAlive() {
        return process.isAlive();
    }

    @Override
    public void destroy() {
        process.destroy();
    }

    @Override
    public void destroyForcibly() {
        process.destroyForcibly();
    }
}
