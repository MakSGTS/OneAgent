package com.oneagent.edt.runtime;

import static org.junit.Assert.assertSame;

import java.nio.file.Path;
import org.junit.Assume;
import org.junit.Test;

public final class RuntimeProbeProcessTest {
    @Test
    public void probesThePublicRuntimeTwiceFromTheExplicitFixtureDirectory() throws Exception {
        String executable = System.getenv("ONEAGENT_MCP_EXECUTABLE");
        String fixture = System.getenv("ONEAGENT_MCP_FIXTURE");
        Assume.assumeTrue("ONEAGENT_MCP_EXECUTABLE must select the real-process gate",
                executable != null && !executable.isBlank());
        Assume.assumeTrue("ONEAGENT_MCP_FIXTURE must select the real-process gate",
                fixture != null && !fixture.isBlank());

        Path fixturePath = Path.of(fixture).toAbsolutePath().normalize();
        RuntimeProbe probe = new RuntimeProbe();

        assertSame(ProbeResult.COMPATIBLE,
                probe.probe(executable, fixturePath, CancellationToken.NONE));
        assertSame(ProbeResult.COMPATIBLE,
                probe.probe(executable, fixturePath, CancellationToken.NONE));
    }
}
