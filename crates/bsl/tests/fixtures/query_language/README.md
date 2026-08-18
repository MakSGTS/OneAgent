# Query-Language Evidence Fixtures

This directory contains raw 1C query-language text for the parser investigation
required by ADR-0021. Files do not contain BSL assignments, `New Query` wrappers,
or multiline BSL continuation markers. Fixture order has no semantic meaning.

## Manifest

| Fixture | Provenance | Evidence origin | Expected first-slice classification | Expected diagnostic |
|---|---|---|---|---|
| `accepted_catalog_en.query` | Copied from an existing repository test | `crates/bsl/src/queries.rs:455-474` | Accepted: one English `SELECT`, one direct `Catalog.Products` source | None |
| `accepted_catalog_ru.query` | Copied from an existing repository test | `crates/bsl/src/queries.rs:477-496` | Accepted: one Russian `ВЫБРАТЬ`, one direct `Справочник.Номенклатура` source | None |
| `accepted_information_register_en.query` | Verbatim decoded query text from a real one-line BSL string | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:764-771` | Accepted: one direct `InformationRegister.ObjectsToDelete` source with alias `Tab` | None |
| `unsupported_parameter_source_en.query` | Verbatim decoded query text from a real one-line BSL string | `OneAgent_EDTproject/src/DataProcessors/ExportImportEnterpriseData/ObjectModule.bsl:871-877` | Unsupported: `&MetadataTableName` is a parameter-supplied source | External or parameter data source |

The one-line literals can be decoded without inventing multiline continuation
behavior: their enclosing BSL quotes are excluded and their contents are kept
unchanged. The parameter-source fixture is evidence of query-template syntax; it
is not evidence that the current `BslQuery` extractor creates a Query node for
that template.

## Multiline fixture decision

No multiline fixture was added by the focused decoding investigation. The
official [1C:Enterprise 8.3.27 Developer Guide](https://1c-dn.com/download-trial/files/guides/developer_guide.pdf)
confirms multiline string constants, continuation lines beginning with `|`, and
doubled-quote decoding. The official [line-wrapping standard](https://kb.1ci.com/1C_Enterprise_Platform/Guides/Developer_Guides/1C_Enterprise_Development_Standards/Code_conventions/Using_1C_Enterprise_language_structures/Line_wrapping/)
shows indented continuation markers. Neither source defines the exact runtime
contribution of the marker or preceding indentation, newline code units,
LF/CRLF behavior, empty continuation lines, or decoded-to-BSL source mapping.

The complete direct constructor at
`OneAgent_EDTproject/src/CommonModules/wms_mobile_ProductsPicking/Module.bsl:397-479`
and the complete returned program at
`OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:169-359`
remain the preferred repository-owned candidates after that contract is proven.
Until then, deriving `.query` bytes by stripping `|` prefixes would not be a
reproducible source-preserving transformation.

## Deliberate evidence gaps

No raw fixture currently represents `JOIN`, `UNION`, a nested query, a batch,
temporary tables, virtual tables, scalar parameters outside source positions,
query comments, keyword-like query string literals, or dynamically replaced
multiline text. Real repository evidence for these forms is encoded inside BSL
multiline strings, while the current extractor defines no multiline decoding
contract. Removing `|` prefixes would be a speculative transformation. This
blocks raw negative fixtures for `JOIN`, `UNION`, nested queries, batches,
temporary tables, and virtual tables separately; the source structures are
Confirmed, but their decoded raw programs remain Unknown.

Malformed static query text is Accepted by ADR-0021 as a required diagnostic
case, but no repository-owned malformed raw input was established. Its fixture
therefore remains an evidence gap rather than guessed syntax. The Russian
metadata-type spelling `РегистрСведений` is repository-backed outside query
text, but its query-language form and fixture remain Unknown. Case-variant
fixtures also remain Unknown pending repository evidence.

See `docs/architecture/query-language-parser-investigation.md` for the complete
Confirmed, Accepted, and Unknown classification, source-location blocker, and
diagnostic taxonomy.
