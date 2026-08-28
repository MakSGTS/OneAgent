import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import java.util.jar.Attributes;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.jar.Manifest;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilderFactory;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

public final class VerifyPackage {
    private static final String VERSION = "0.1.0.202608270000";
    private static final Set<String> REPOSITORY_FILES = Set.of(
            "artifacts.jar",
            "artifacts.xml.xz",
            "content.jar",
            "content.xml.xz",
            "features/com.oneagent.edt.feature_" + VERSION + ".jar",
            "p2.index",
            "plugins/com.oneagent.edt_" + VERSION + ".jar");
    private static final Set<String> IMPORTS = Set.of(
            "org.eclipse.core.commands",
            "org.eclipse.core.resources",
            "org.eclipse.core.runtime",
            "org.eclipse.core.runtime.jobs",
            "org.eclipse.core.runtime.preferences",
            "org.eclipse.jface.dialogs",
            "org.eclipse.jface.preference",
            "org.eclipse.jface.viewers",
            "org.eclipse.swt.widgets",
            "org.eclipse.ui",
            "org.eclipse.ui.handlers",
            "org.eclipse.ui.plugin",
            "org.osgi.framework",
            "org.osgi.service.prefs");
    private static final Set<String> CONTENT_UNITS = Set.of(
            "202608270000.com.oneagent.edt.category",
            "com.oneagent.edt",
            "com.oneagent.edt.feature.feature.group",
            "com.oneagent.edt.feature.feature.jar");
    private static final Set<String> EXTENSION_POINTS = Set.of(
            "org.eclipse.ui.commands",
            "org.eclipse.ui.handlers",
            "org.eclipse.ui.menus",
            "org.eclipse.ui.preferencePages",
            "org.eclipse.core.runtime.preferences");
    private static final Set<String> TEST_SUITES = Set.of(
            "com.oneagent.edt.EclipseHostTest",
            "com.oneagent.edt.ProbeControllerTest",
            "com.oneagent.edt.ProbeRuntimeHandlerTest",
            "com.oneagent.edt.ProjectEligibilityTest",
            "com.oneagent.edt.RuntimeExecutableTest",
            "com.oneagent.edt.runtime.RuntimeProbeProcessTest",
            "com.oneagent.edt.runtime.RuntimeProbeTest",
            "com.oneagent.edt.runtime.StrictJsonParserTest");
    private static final List<String> SENSITIVE_TEXT = List.of(
            "/Users/", "C:\\Users\\", ".p2/pool", "services.1c.dev",
            "ONEC_ITS_PASSWORD", "<password>");

    private VerifyPackage() {
    }

    public static void main(String[] args) throws Exception {
        Path repository = args.length == 0
                ? Path.of("repositories/com.oneagent.edt.repository/target/repository")
                : Path.of(args[0]);
        repository = repository.toAbsolutePath().normalize();
        require(Files.isDirectory(repository), "repository directory is missing: " + repository);

        verifyTestReports(Path.of("tests/com.oneagent.edt.tests/target/surefire-reports"));
        Set<String> actualFiles = new TreeSet<>();
        try (var paths = Files.walk(repository)) {
            paths.filter(Files::isRegularFile)
                    .map(repository::relativize)
                    .map(path -> path.toString().replace('\\', '/'))
                    .forEach(actualFiles::add);
        }
        require(actualFiles.equals(REPOSITORY_FILES),
                "unexpected repository inventory: " + actualFiles);

        Path feature = repository.resolve("features/com.oneagent.edt.feature_" + VERSION + ".jar");
        Path bundle = repository.resolve("plugins/com.oneagent.edt_" + VERSION + ".jar");
        verifyFeature(feature);
        verifyBundle(bundle);
        verifyContent(repository.resolve("content.jar"));
        verifyArtifacts(repository.resolve("artifacts.jar"));
        scanRepositoryText(repository, feature, bundle);

        System.out.println("tests=39/failures=0/errors=0/skipped=0");
        System.out.println("repository-files=" + actualFiles.size());
        System.out.println("content-units=" + CONTENT_UNITS.size());
        System.out.println("feature=com.oneagent.edt.feature/" + VERSION);
        System.out.println("bundle=com.oneagent.edt/" + VERSION + "/JavaSE-17/class-major-61");
        System.out.println("result=PASS");
    }

    private static void verifyFeature(Path feature) throws Exception {
        try (JarFile jar = new JarFile(feature.toFile())) {
            Set<String> entries = entryNames(jar);
            require(entries.equals(Set.of("META-INF/", "META-INF/MANIFEST.MF", "feature.xml")),
                    "unexpected feature JAR inventory: " + entries);
            Document xml = parse(read(jar, "feature.xml"));
            Element root = xml.getDocumentElement();
            require("com.oneagent.edt.feature".equals(root.getAttribute("id")), "wrong feature id");
            require(VERSION.equals(root.getAttribute("version")), "wrong feature version");
            NodeList plugins = root.getElementsByTagName("plugin");
            require(plugins.getLength() == 1, "feature must contain exactly one plug-in");
            Element plugin = (Element) plugins.item(0);
            require("com.oneagent.edt".equals(plugin.getAttribute("id")), "wrong feature plug-in");
            require(VERSION.equals(plugin.getAttribute("version")), "wrong feature plug-in version");
            NodeList licenses = root.getElementsByTagName("license");
            require(licenses.getLength() == 1
                    && "https://www.apache.org/licenses/LICENSE-2.0"
                            .equals(((Element) licenses.item(0)).getAttribute("url")),
                    "feature must declare Apache-2.0");
        }
    }

    private static void verifyBundle(Path bundle) throws Exception {
        try (JarFile jar = new JarFile(bundle.toFile())) {
            Manifest manifest = new Manifest(new ByteArrayInputStream(read(jar, "META-INF/MANIFEST.MF")));
            Attributes attributes = manifest.getMainAttributes();
            require("com.oneagent.edt;singleton:=true"
                    .equals(attributes.getValue("Bundle-SymbolicName")), "wrong bundle id");
            require(VERSION.equals(attributes.getValue("Bundle-Version")), "wrong bundle version");
            require("17".equals(attributes.getValue("Java-Version")), "wrong Java release marker");
            require("25".equals(attributes.getValue("Build-Jdk-Spec")), "wrong build JDK marker");
            require("osgi.ee;filter:=\"(&(osgi.ee=JavaSE)(version=17))\""
                    .equals(attributes.getValue("Require-Capability")),
                    "wrong bundle execution environment capability");
            require(attributes.getValue("Export-Package") == null, "production bundle must export no package");
            require(attributes.getValue("Bundle-ClassPath") == null,
                    "production bundle must not embed a library class path");
            require(importNames(attributes.getValue("Import-Package")).equals(IMPORTS),
                    "unexpected production imports: " + importNames(attributes.getValue("Import-Package")));

            Set<String> entries = entryNames(jar);
            require(entries.contains("plugin.xml"), "plugin.xml is missing");
            require(entries.stream().noneMatch(VerifyPackage::isForbiddenBundleEntry),
                    "test, native, Runtime, JRE, or JavaFX content is packaged");
            for (String entry : entries) {
                if (entry.endsWith(".class")) {
                    require(classMajor(read(jar, entry)) == 61, "non-Java-17 bytecode: " + entry);
                }
            }

            Document plugin = parse(read(jar, "plugin.xml"));
            Set<String> points = new HashSet<>();
            NodeList extensions = plugin.getElementsByTagName("extension");
            for (int index = 0; index < extensions.getLength(); index++) {
                points.add(((Element) extensions.item(index)).getAttribute("point"));
            }
            require(points.equals(EXTENSION_POINTS), "unexpected extension points: " + points);
            require(hasElementWithAttribute(plugin, "command", "id",
                    "com.oneagent.edt.commands.probeRuntime"), "command contribution is missing");
            require(hasElementWithAttribute(plugin, "page", "id",
                    "com.oneagent.edt.preferences"), "preference page is missing");
        }
    }

    private static void verifyContent(Path contentJar) throws Exception {
        try (JarFile jar = new JarFile(contentJar.toFile())) {
            Document content = parse(read(jar, "content.xml"));
            Set<String> units = attributeValues(content, "unit", "id");
            require(units.equals(CONTENT_UNITS), "unexpected p2 content units: " + units);
        }
    }

    private static void verifyTestReports(Path reports) throws Exception {
        require(Files.isDirectory(reports), "Surefire reports are missing: " + reports);
        List<Path> files;
        try (var paths = Files.list(reports)) {
            files = paths.filter(path -> path.getFileName().toString().startsWith("TEST-"))
                    .filter(path -> path.getFileName().toString().endsWith(".xml"))
                    .sorted()
                    .toList();
        }
        require(files.size() == TEST_SUITES.size(), "unexpected Surefire report count: " + files.size());
        Set<String> suites = new HashSet<>();
        int tests = 0;
        int failures = 0;
        int errors = 0;
        int skipped = 0;
        for (Path file : files) {
            Element suite = parse(Files.readAllBytes(file)).getDocumentElement();
            suites.add(suite.getAttribute("name"));
            tests += integerAttribute(suite, "tests");
            failures += integerAttribute(suite, "failures");
            errors += integerAttribute(suite, "errors");
            skipped += integerAttribute(suite, "skipped");
        }
        require(suites.equals(TEST_SUITES), "unexpected Surefire suites: " + suites);
        require(tests == 39 && failures == 0 && errors == 0 && skipped == 0,
                "incomplete test gate: tests=" + tests + ", failures=" + failures
                        + ", errors=" + errors + ", skipped=" + skipped);
    }

    private static void verifyArtifacts(Path artifactsJar) throws Exception {
        try (JarFile jar = new JarFile(artifactsJar.toFile())) {
            Document artifacts = parse(read(jar, "artifacts.xml"));
            Set<String> ids = attributeValues(artifacts, "artifact", "id");
            require(ids.equals(Set.of("com.oneagent.edt", "com.oneagent.edt.feature")),
                    "unexpected p2 artifacts: " + ids);
        }
    }

    private static void scanRepositoryText(Path repository, Path feature, Path bundle) throws Exception {
        List<String> text = new ArrayList<>();
        text.add(Files.readString(repository.resolve("p2.index"), StandardCharsets.UTF_8));
        for (Path jarPath : List.of(repository.resolve("content.jar"),
                repository.resolve("artifacts.jar"), feature, bundle)) {
            try (JarFile jar = new JarFile(jarPath.toFile())) {
                for (JarEntry entry : jar.stream().filter(item -> !item.isDirectory()).toList()) {
                    String name = entry.getName();
                    if (name.endsWith(".xml") || name.endsWith(".MF")
                            || name.endsWith(".properties") || name.endsWith("pom.xml")) {
                        text.add(new String(read(jar, name), StandardCharsets.UTF_8));
                    }
                }
            }
        }
        String joined = String.join("\n", text);
        for (String forbidden : SENSITIVE_TEXT) {
            require(!joined.contains(forbidden), "sensitive or local text is packaged: " + forbidden);
        }
    }

    private static Set<String> importNames(String value) {
        require(value != null, "Import-Package is missing");
        Set<String> names = new HashSet<>();
        for (String item : value.split(",")) {
            names.add(item.strip().split(";", 2)[0]);
        }
        return names;
    }

    private static boolean isForbiddenBundleEntry(String name) {
        String lower = name.toLowerCase();
        return lower.contains("/test/") || lower.contains("/tests/")
                || lower.endsWith(".exe") || lower.endsWith(".dylib")
                || lower.endsWith(".so") || lower.endsWith(".dll")
                || lower.contains("javafx") || lower.contains("oneagent-mcp")
                || lower.startsWith("jre/") || lower.startsWith("runtime/");
    }

    private static int classMajor(byte[] bytes) throws IOException {
        try (DataInputStream input = new DataInputStream(new ByteArrayInputStream(bytes))) {
            require(input.readInt() == 0xCAFEBABE, "invalid class file");
            input.readUnsignedShort();
            return input.readUnsignedShort();
        }
    }

    private static Set<String> entryNames(JarFile jar) {
        Set<String> names = new HashSet<>();
        jar.stream().map(JarEntry::getName).forEach(names::add);
        return names;
    }

    private static byte[] read(JarFile jar, String name) throws IOException {
        JarEntry entry = jar.getJarEntry(name);
        require(entry != null, "missing JAR entry: " + name);
        try (InputStream input = jar.getInputStream(entry)) {
            return input.readAllBytes();
        }
    }

    private static Document parse(byte[] bytes) throws Exception {
        DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
        factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
        factory.setFeature("http://xml.org/sax/features/external-general-entities", false);
        factory.setFeature("http://xml.org/sax/features/external-parameter-entities", false);
        factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "");
        factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");
        return factory.newDocumentBuilder().parse(new ByteArrayInputStream(bytes));
    }

    private static Set<String> attributeValues(Document document, String tag, String attribute) {
        Set<String> values = new HashSet<>();
        NodeList elements = document.getElementsByTagName(tag);
        for (int index = 0; index < elements.getLength(); index++) {
            values.add(((Element) elements.item(index)).getAttribute(attribute));
        }
        return values;
    }

    private static int integerAttribute(Element element, String attribute) {
        try {
            return Integer.parseInt(element.getAttribute(attribute));
        } catch (NumberFormatException error) {
            throw new IllegalStateException("invalid integer attribute: " + attribute, error);
        }
    }

    private static boolean hasElementWithAttribute(
            Document document, String tag, String attribute, String value) {
        return attributeValues(document, tag, attribute).contains(value);
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new IllegalStateException(message);
        }
    }
}
