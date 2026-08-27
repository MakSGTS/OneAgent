package com.oneagent.edt;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.lang.reflect.Proxy;
import java.net.URI;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.resources.IResource;
import org.eclipse.core.runtime.CoreException;
import org.eclipse.core.runtime.IAdaptable;
import org.eclipse.core.runtime.Status;
import org.eclipse.jface.viewers.StructuredSelection;
import org.junit.Test;

public final class ProjectEligibilityTest {
    private static final Path LOCATION = Path.of(System.getProperty("java.io.tmpdir"))
            .toAbsolutePath().normalize();

    @Test
    public void acceptsExactlyOneAdaptedEligibleProject() {
        FakeProject project = new FakeProject();
        assertEquals(Optional.of(LOCATION), resolve(List.of("selection"), project, true));
        assertTrue(resolve(List.of(), project, true).isEmpty());
        assertTrue(resolve(List.of("first", "second"), project, true).isEmpty());
        assertTrue(ProjectEligibility.resolve(
                List.of("selection"),
                ignored -> {
                    throw new SecurityException("private project");
                },
                ignored -> true).isEmpty());
        assertTrue(ProjectEligibility.resolve(null).isEmpty());
    }

    @Test
    public void rejectsEveryProjectStateAndInspectionFailure() {
        FakeProject project = new FakeProject();

        project.exists = false;
        assertEmpty(project);
        project = new FakeProject();
        project.open = false;
        assertEmpty(project);
        project = new FakeProject();
        project.accessible = false;
        assertEmpty(project);
        project = new FakeProject();
        project.linked = true;
        assertEmpty(project);
        project = new FakeProject();
        project.virtual = true;
        assertEmpty(project);
        project = new FakeProject();
        project.nature = false;
        assertEmpty(project);
        project = new FakeProject();
        project.natureFailure = true;
        assertEmpty(project);
    }

    @Test
    public void rejectsMissingRemoteRelativeUnnormalizedAndUnreadableLocations() {
        FakeProject project = new FakeProject();
        project.location = null;
        assertEmpty(project);
        project = new FakeProject();
        project.locationUri = null;
        assertEmpty(project);
        project = new FakeProject();
        project.locationUri = URI.create("memory:/workspace");
        assertEmpty(project);
        project = new FakeProject();
        project.location = Path.of("relative");
        assertEmpty(project);
        project = new FakeProject();
        project.location = LOCATION.resolve("child").resolve("..");
        assertEmpty(project);
        assertTrue(resolve(List.of("selection"), new FakeProject(), false).isEmpty());
    }

    @Test
    public void productionAdapterAcceptsDirectAndAdaptableProjectsButRejectsResources() {
        IProject project = projectProxy();
        assertEquals(Optional.of(LOCATION),
                ProjectEligibility.resolve(new StructuredSelection(project)));
        IAdaptable adaptable = new IAdaptable() {
            @Override
            public <T> T getAdapter(Class<T> adapter) {
                return adapter == IProject.class ? adapter.cast(project) : null;
            }
        };
        assertEquals(Optional.of(LOCATION),
                ProjectEligibility.resolve(new StructuredSelection(adaptable)));

        IResource resource = (IResource) Proxy.newProxyInstance(
                IResource.class.getClassLoader(),
                new Class<?>[] {IResource.class},
                (proxy, method, arguments) ->
                    method.getName().equals("getAdapter") ? project : defaultValue(method.getReturnType()));
        assertTrue(ProjectEligibility.resolve(new StructuredSelection(resource)).isEmpty());
    }

    private static Optional<Path> resolve(List<?> elements, FakeProject project, boolean readable) {
        return ProjectEligibility.resolve(elements, ignored -> project, ignored -> readable);
    }

    private static void assertEmpty(FakeProject project) {
        assertTrue(resolve(List.of("selection"), project, true).isEmpty());
    }

    private static IProject projectProxy() {
        return (IProject) Proxy.newProxyInstance(
                IProject.class.getClassLoader(),
                new Class<?>[] {IProject.class},
                (proxy, method, arguments) -> switch (method.getName()) {
                    case "exists", "isOpen", "isAccessible" -> true;
                    case "isLinked", "isVirtual" -> false;
                    case "hasNature" -> ProjectEligibility.EDT_CONFIGURATION_NATURE.equals(arguments[0]);
                    case "getLocation" -> org.eclipse.core.runtime.Path.fromOSString(LOCATION.toString());
                    case "getLocationURI" -> LOCATION.toUri();
                    case "toString" -> "project";
                    default -> defaultValue(method.getReturnType());
                });
    }

    private static Object defaultValue(Class<?> type) {
        if (!type.isPrimitive()) {
            return null;
        }
        if (type == boolean.class) {
            return false;
        }
        if (type == char.class) {
            return '\0';
        }
        return 0;
    }

    private static final class FakeProject implements ProjectEligibility.ProjectView {
        private boolean exists = true;
        private boolean open = true;
        private boolean accessible = true;
        private boolean linked;
        private boolean virtual;
        private boolean nature = true;
        private boolean natureFailure;
        private Path location = LOCATION;
        private URI locationUri = LOCATION.toUri();

        @Override
        public boolean exists() {
            return exists;
        }

        @Override
        public boolean isOpen() {
            return open;
        }

        @Override
        public boolean isAccessible() {
            return accessible;
        }

        @Override
        public boolean isLinked() {
            return linked;
        }

        @Override
        public boolean isVirtual() {
            return virtual;
        }

        @Override
        public boolean hasNature(String natureId) throws CoreException {
            if (natureFailure) {
                throw new CoreException(Status.error("nature failure"));
            }
            return nature && ProjectEligibility.EDT_CONFIGURATION_NATURE.equals(natureId);
        }

        @Override
        public Path location() {
            return location;
        }

        @Override
        public URI locationUri() {
            return locationUri;
        }
    }
}
