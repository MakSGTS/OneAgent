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

## Deliberate evidence gaps

No raw fixture currently represents `JOIN`, `UNION`, a nested query, a batch,
temporary tables, virtual tables, scalar parameters outside source positions,
query comments, keyword-like query string literals, or dynamically replaced
multiline text. Real repository evidence for these forms is encoded inside BSL
multiline strings, while the current extractor defines no multiline decoding
contract. Removing `|` prefixes would be a speculative transformation.

Malformed static query text is Accepted by ADR-0021 as a required diagnostic
case, but no repository-owned malformed raw input was established. Its fixture
therefore remains an evidence gap rather than guessed syntax. The Russian
metadata-type spelling `РегистрСведений` is repository-backed outside query
text, but its query-language form and fixture remain Unknown. Case-variant
fixtures also remain Unknown pending repository evidence.

See `docs/architecture/query-language-parser-investigation.md` for the complete
Confirmed, Accepted, and Unknown classification, source-location blocker, and
diagnostic taxonomy.
