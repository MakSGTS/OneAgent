import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.jar.Attributes;
import java.util.jar.Manifest;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilderFactory;
import org.w3c.dom.Document;
import org.w3c.dom.NodeList;

public final class VerifyHostBoundary {
    private static final String BUILD_JAVA = "25";
    private static final String BUNDLE_JAVA = "17";
    private static final String EXECUTION_ENVIRONMENT = "JavaSE-17";

    private VerifyHostBoundary() {
    }

    public static void main(String[] args) throws Exception {
        Path edtRoot = args.length == 0 ? Path.of(".") : Path.of(args[0]);
        edtRoot = edtRoot.toAbsolutePath().normalize();
        require(Files.isDirectory(edtRoot), "EDT root is missing: " + edtRoot);

        require(BUILD_JAVA.equals(System.getProperty("java.specification.version")),
                "EDT build must run on JDK " + BUILD_JAVA + ", got "
                        + System.getProperty("java.specification.version"));

        verifyPom(edtRoot.resolve("pom.xml"));
        verifyManifest(edtRoot.resolve("bundles/com.oneagent.edt/META-INF/MANIFEST.MF"));
        verifyManifest(edtRoot.resolve("tests/com.oneagent.edt.tests/META-INF/MANIFEST.MF"));
        verifyDocumentation(edtRoot.resolve("README.md"));
        verifyCi(edtRoot.resolve("../../.github/workflows/ci.yml").normalize());

        System.out.println("build-jdk=" + BUILD_JAVA);
        System.out.println("bundle-java=" + BUNDLE_JAVA);
        System.out.println("edt-host=x86_64/JDK-17/OpenJFX-17");
        System.out.println("result=PASS");
    }

    private static void verifyPom(Path pom) throws Exception {
        Document document = parse(Files.readAllBytes(pom));
        require(BUNDLE_JAVA.equals(singleText(document, "maven.compiler.release")),
                "maven.compiler.release must remain " + BUNDLE_JAVA);
        require(EXECUTION_ENVIRONMENT.equals(singleText(document, "executionEnvironment")),
                "Tycho execution environment must remain " + EXECUTION_ENVIRONMENT);
        require(BUNDLE_JAVA.equals(singleText(document, "release")),
                "Tycho compiler release must remain " + BUNDLE_JAVA);
    }

    private static void verifyManifest(Path path) throws Exception {
        Manifest manifest = new Manifest(new ByteArrayInputStream(Files.readAllBytes(path)));
        Attributes attributes = manifest.getMainAttributes();
        require(EXECUTION_ENVIRONMENT.equals(
                attributes.getValue("Bundle-RequiredExecutionEnvironment")),
                path + " must require " + EXECUTION_ENVIRONMENT);
    }

    private static void verifyDocumentation(Path readme) throws Exception {
        String text = Files.readString(readme, StandardCharsets.UTF_8);
        require(text.contains("Build JDK: Temurin JDK 25"),
                "README must document the JDK 25 build boundary");
        require(text.contains("x86_64 EDT 2026.1 on an x86_64 JDK 17"),
                "README must document the x86_64 EDT/JDK 17 host boundary");
        require(text.contains("OpenJFX 17 modules available to the host"),
                "README must document the OpenJFX 17 host boundary");
        require(text.contains("A boundary change requires new disposable host evidence"),
                "README must document the boundary change policy");
    }

    private static void verifyCi(Path workflow) throws Exception {
        String text = Files.readString(workflow, StandardCharsets.UTF_8);
        require(text.contains("java-version: 25"),
                "CI must configure JDK " + BUILD_JAVA + " for the EDT build");
        require(text.contains("java scripts/VerifyHostBoundary.java"),
                "CI must run the EDT host boundary guard");
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

    private static String singleText(Document document, String elementName) {
        NodeList elements = document.getElementsByTagName(elementName);
        require(elements.getLength() == 1,
                "expected exactly one " + elementName + " element, got " + elements.getLength());
        return elements.item(0).getTextContent().trim();
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new IllegalStateException(message);
        }
    }
}
