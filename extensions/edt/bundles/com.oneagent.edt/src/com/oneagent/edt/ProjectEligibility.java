package com.oneagent.edt;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.resources.IResource;
import org.eclipse.core.runtime.CoreException;
import org.eclipse.core.runtime.IAdaptable;
import org.eclipse.jface.viewers.IStructuredSelection;

final class ProjectEligibility {
    static final String EDT_CONFIGURATION_NATURE =
            "com._1c.g5.v8.dt.core.V8ConfigurationNature";

    private static final DirectoryAccess SYSTEM_DIRECTORIES =
            path -> Files.isDirectory(path) && Files.isReadable(path);

    private ProjectEligibility() {
    }

    static Optional<Path> resolve(IStructuredSelection selection) {
        if (selection == null) {
            return Optional.empty();
        }
        return resolve(selection.toList(), ProjectEligibility::adapt, SYSTEM_DIRECTORIES);
    }

    static Optional<Path> resolve(List<?> elements, ProjectAdapter adapter,
            DirectoryAccess directories) {
        if (elements == null || elements.size() != 1 || adapter == null || directories == null) {
            return Optional.empty();
        }
        try {
            ProjectView project = adapter.adapt(elements.get(0));
            if (project == null) {
                return Optional.empty();
            }
            if (!project.exists() || !project.isOpen() || !project.isAccessible()
                    || project.isLinked() || project.isVirtual()
                    || !project.hasNature(EDT_CONFIGURATION_NATURE)) {
                return Optional.empty();
            }
            Path location = project.location();
            URI locationUri = project.locationUri();
            if (location == null || locationUri == null || locationUri.getScheme() == null
                    || !"file".equalsIgnoreCase(locationUri.getScheme())
                    || !location.isAbsolute()) {
                return Optional.empty();
            }
            Path normalized = location.normalize();
            if (!location.equals(normalized) || !directories.isReadableDirectory(normalized)) {
                return Optional.empty();
            }
            return Optional.of(normalized);
        } catch (CoreException | RuntimeException error) {
            return Optional.empty();
        }
    }

    private static ProjectView adapt(Object element) {
        if (element instanceof IResource && !(element instanceof IProject)) {
            return null;
        }
        IProject project = element instanceof IProject direct
                ? direct
                : element instanceof IAdaptable adaptable
                        ? adaptable.getAdapter(IProject.class)
                        : null;
        return project == null ? null : new EclipseProjectView(project);
    }

    interface ProjectAdapter {
        ProjectView adapt(Object element);
    }

    interface DirectoryAccess {
        boolean isReadableDirectory(Path path);
    }

    interface ProjectView {
        boolean exists();

        boolean isOpen();

        boolean isAccessible();

        boolean isLinked();

        boolean isVirtual();

        boolean hasNature(String natureId) throws CoreException;

        Path location();

        URI locationUri();
    }

    private record EclipseProjectView(IProject project) implements ProjectView {
        @Override
        public boolean exists() {
            return project.exists();
        }

        @Override
        public boolean isOpen() {
            return project.isOpen();
        }

        @Override
        public boolean isAccessible() {
            return project.isAccessible();
        }

        @Override
        public boolean isLinked() {
            return project.isLinked();
        }

        @Override
        public boolean isVirtual() {
            return project.isVirtual();
        }

        @Override
        public boolean hasNature(String natureId) throws CoreException {
            return project.hasNature(natureId);
        }

        @Override
        public Path location() {
            return project.getLocation() == null ? null : project.getLocation().toFile().toPath();
        }

        @Override
        public URI locationUri() {
            return project.getLocationURI();
        }
    }
}
