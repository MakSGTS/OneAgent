package com.oneagent.edt.runtime;

import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

interface ProbeProcess {
    OutputStream stdin();

    InputStream stdout();

    InputStream stderr();

    boolean waitFor(long timeout, TimeUnit unit) throws InterruptedException;

    int exitValue();

    boolean isAlive();

    void destroy();

    void destroyForcibly();

    interface Factory {
        ProbeProcess start(String executable, Path workingDirectory) throws Exception;
    }
}
