# OneAgent Designer XML Source Corpus

## Status

Repository-local registration of a real hierarchical Designer XML export used
as Sprint 14 source-format and cross-adapter evidence. The complete export is
ignored and is not a CI dependency.

## Registration

The corpus was produced and validated on 2026-08-21:

| Property | Value |
|---|---|
| Local root | `OneAgent_DesignerXML/` |
| Paired EDT root | `OneAgent_EDTproject/` |
| Configuration name | `DNSWorldEdition` |
| Configuration UUID | `408a41e7-907a-4fb3-8999-83d1e8b6e093` |
| Configuration version | `1.22.1.1` |
| Script variant | English |
| Compatibility mode | `Version8_3_21` |
| EDT runtime declaration | `8.3.27` |
| Designer producer | `1C:Enterprise 8.3.27.2214` |
| Dump marker | `ConfigDumpInfo.xml` |
| Dump format | `Hierarchical` |
| XML format version | `2.20` |

`OneAgent` is the repository corpus label. The source project's Eclipse name
is `DNS_WE`, and the configuration's canonical source name is
`DNSWorldEdition`.

## Provenance chain

No XML was synthesized or reconstructed from inferred EDT shapes.

1. The EDT Starter registration binds `OneAgent_EDTproject/` to the installed
   official 1C:EDT 2026.1 product. Its 2026.1.2.2 CLI exported the project
   through `export --project ... --configuration-files ...`, producing 10,974
   files, 234,236,155 bytes, and composite tree SHA-256
   `c3a10585ea63a1fec07538fa2fb8169a53d1cd40eb5877a2f7eb3dbc50649900`.
2. Designer 8.3.27.2214 loaded that intermediate tree into a new disposable
   file infobase:

   ```text
   /opt/1cv8/8.3.27.2214/1cv8 DESIGNER \
     /F /private/tmp/oneagent-designer-ib-2026-8327-20260821 \
     /LoadConfigFromFiles /private/tmp/oneagent-edt-export-20260821 \
     /DisableStartupDialogs /DisableStartupMessages \
     /Out /private/tmp/oneagent-designer-load-2026-8327-20260821.log
   ```

3. Designer, not EDT or repository code, produced the registered corpus with
   an explicit hierarchical dump:

   ```text
   /opt/1cv8/8.3.27.2214/1cv8 DESIGNER \
     /F /private/tmp/oneagent-designer-ib-2026-8327-20260821 \
     /DumpConfigToFiles /private/tmp/oneagent-designer-xml-export-2026-20260821 \
     -Format Hierarchical \
     /DisableStartupDialogs /DisableStartupMessages \
     /Out /private/tmp/oneagent-designer-dump-2026-8327-20260821.log
   ```

4. The resulting tree was copied without content changes to
   `OneAgent_DesignerXML/`.

The intermediate EDT export is not registered as a Designer corpus because it
has no `ConfigDumpInfo.xml`. The registered tree contains the marker generated
by Designer and is therefore direct evidence for Designer project detection,
artifact discovery, and assembly. A control export with EDT 2025.1.5.34 kept
the same root `Configuration.xml` but produced a different full tree and its
subsequent Designer dump omitted the parent-configuration artifacts. It was
therefore rejected in favor of the EDT version registered for this project.

## Inventory and hashes

The registered tree contains 10,964 files and 237,458,744 file-content bytes:

| Artifact class | Count |
|---|---:|
| XML files | 7,585 |
| BSL modules | 2,717 |
| Other binary or text artifacts | 662 |
| Top-level directories | 40 |

Root and tree hashes:

| Artifact | SHA-256 |
|---|---|
| `OneAgent_EDTproject/src/` composite tree | `ceb8658c8cd26432197f83873e14295d165d5b0bab04cbdb2df9613a8fd3a42f` |
| Intermediate official EDT XML composite tree | `c3a10585ea63a1fec07538fa2fb8169a53d1cd40eb5877a2f7eb3dbc50649900` |
| `OneAgent_DesignerXML/` composite tree | `2579b567644c4ab2f57fad5357c4e1619423e7fcc0de9d7f783c945925245eac` |
| `Configuration.xml` | `b7eed83a154d0f68c858f10d991ee985fb6d7df878f7abb328c1e441d57a2bdd` |
| `ConfigDumpInfo.xml` | `b0163f453ca4df2674aac40bbb7ae66e66e67d80bbb9aaf2a0f4e9592898339c` |
| `Ext/ParentConfigurations/StandardSubsystemsLibrary.cf` | `e443fbab718c7f50e55dbcd53b23699a274dd95f22dff7bda1ae8fefd1789402` |
| `Ext/ParentConfigurations.bin` | `7cc904982722bfcb7006214e046a09622f7658f5ce1472446fa86dfb9e4a3e71` |

Each composite tree hash is the SHA-256 of the canonical text stream produced
by this command from the corresponding root:

```text
find . -type f -print0 \
  | LC_ALL=C sort -z \
  | xargs -0 shasum -a 256 \
  | shasum -a 256
```

The stream includes each file's `./`-relative path and content hash. Directory
timestamps, permissions, and empty directories are deliberately excluded.

## Paired identity evidence

The EDT and Designer roots have different serialization layouts but retain the
same source identity:

| Dimension | EDT | Designer XML | Result |
|---|---|---|---|
| Configuration UUID | `408a41e7-907a-4fb3-8999-83d1e8b6e093` | `408a41e7-907a-4fb3-8999-83d1e8b6e093` | equal |
| Configuration name | `DNSWorldEdition` | `DNSWorldEdition` | equal |
| Configuration version | `1.22.1.1` | `1.22.1.1` | equal |
| Compatibility mode | `8.3.21` | `Version8_3_21` | equivalent source spellings |
| Parent configuration CF SHA-256 | `e443fbab718c7f50e55dbcd53b23699a274dd95f22dff7bda1ae8fefd1789402` | `e443fbab718c7f50e55dbcd53b23699a274dd95f22dff7bda1ae8fefd1789402` | equal |
| Web Service `Exchange` module, BOM/line-ending normalized SHA-256 | `27acb605034d7a6d5fdf7663d35b2da1b74e2ac61d814cdc0432b83f7364d191` | `27acb605034d7a6d5fdf7663d35b2da1b74e2ac61d814cdc0432b83f7364d191` | equal |
| HTTP Service `Site` module, BOM/line-ending normalized SHA-256 | `c6d73a7da06f3e0b2c9509b5170004c0f5f5873e4a6a5aef6a4ac859e058f0d6` | `c6d73a7da06f3e0b2c9509b5170004c0f5f5873e4a6a5aef6a4ac859e058f0d6` | equal |

The normalized module hash removes a UTF-8 BOM from the Designer artifact and
normalizes CRLF to LF. Raw file hashes must still be used for fixture
provenance because the normalization is only an identity observation.

Sprint 14 must define its complete canonical conformance projection before
claiming adapter equivalence. The table above proves that the paired corpus is
non-empty and identity-related; it does not pre-accept all payload, request, or
terminal-outcome equivalence.

## Known bridge loss boundary

The EDT-to-Designer load completed with exit code zero but Designer reported
four source event bindings that it did not load because they are not events of
the corresponding elements:

- `CommonForms/AllowedSubdivisions/Ext/Form.xml`:
  `Item.AllowedSubdivisionsSubdivision.StartChoice`;
- `InformationRegisters/DescriptionOfDiscountedProducts/Forms/`
  `FormOfDescriptionOfDiscountedProducts/Ext/Form.xml`:
  `Form.OnReadAtServer`, `Form.BeforeWriteAtServer`, and
  `Form.AfterWriteAtServer`.

The same four warnings occurred when Designer 8.3.27 loaded both the EDT
2025.1 control and the EDT 2026.1 authoritative intermediate, and when Designer
8.5.1 loaded the EDT 2026.1 intermediate. The corpus must not be described as
a lossless field-for-field conversion of the EDT project. These invalid event bindings
are outside accepted semantic identity and must be excluded explicitly or
investigated before any later cross-adapter oracle includes form-event payload.
No project source was corrected or rewritten.

## Validation evidence

The final Designer tree passed one full XML import attempt into a second clean
8.3.27.2214 infobase:

```text
/opt/1cv8/8.3.27.2214/ibcmd config import \
  --data=/private/tmp/oneagent-designer-2026-validation.eeLFzM/import-server-data \
  --database-path=/private/tmp/oneagent-designer-2026-validation.eeLFzM/import-infobase \
  /private/tmp/oneagent-designer-2026-validation.eeLFzM/source
```

The command completed with exit code zero and no warnings. Designer then saved
a temporary 140,995,247-byte CF with SHA-256
`ec16ca52c9d5854d3643a120723ad74ce0e753c1d4a1ec334b2d5c6118213667`.
An independently created clean infobase accepted that CF through
`ibcmd config load` with exit code zero. The temporary CF is validation
evidence, not a registered repository artifact.

## Corpus and reduction policy

- Keep `OneAgent_DesignerXML/` ignored. Tests and CI must not require it.
- Treat `ConfigDumpInfo.xml` plus root `Configuration.xml` as the project-root
  evidence for this exact export; Sprint 14 must still investigate which
  markers are required by its accepted detector contract.
- Use raw source paths and hashes from this document when selecting future
  tracked fixtures.
- For parser-local fixtures, exact-copy only the smallest required Designer
  artifacts and record both source and reduced hashes plus every reduction.
- For complete-project or cross-adapter fixtures, create the reduced
  configuration with official 1C tooling and re-export it through Designer;
  do not hand-invent `ConfigDumpInfo.xml`, XML fields, joins, defaults, or
  directory relationships.
- Keep deliberate source-format differences, such as EDT `.mdo` locations,
  Designer `Ext/` directories, BOMs, line endings, and producer-specific
  diagnostics, outside canonical semantic identity unless accepted
  architecture says otherwise.
