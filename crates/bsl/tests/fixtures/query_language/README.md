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
| `unsupported_join_en.query` | Decoded contiguous statement excerpt | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:145-150` | Unsupported: `INNER JOIN` adds another top-level source | Unsupported query structure |
| `unsupported_union_all_en.query` | Decoded complete first statement | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:173-189` | Unsupported: two branches use `UNION ALL` | Unsupported query structure |
| `unsupported_nested_query_en.query` | Decoded complete statement | `OneAgent_EDTproject/src/Reports/AnalyticalReportByCategories/ObjectModule.bsl:145-158` | Unsupported: parenthesized `SELECT` introduces a nested source scope | Unsupported query structure |
| `unsupported_batch_en.query` | Decoded complete two-statement literal | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:134-150` | Unsupported: a delimiter is followed by another `SELECT` statement | Unsupported query structure |
| `unsupported_temporary_table_en.query` | Decoded first-statement excerpt before its delimiter | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:156-163` | Unsupported: `INTO ttProducts` declares a temporary table | Temporary table source |
| `unsupported_virtual_table_en.query` | Decoded complete statement | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:259-273` | Unsupported: `AccumulationRegister.QuantitativeAccounting.Balance(...)` invokes a register virtual table | Virtual table source |

The one-line literals can be decoded without inventing multiline continuation
behavior: their enclosing BSL quotes are excluded and their contents are kept
unchanged. The parameter-source fixture is evidence of query-template syntax; it
is not evidence that the current `BslQuery` extractor creates a Query node for
that template.

## Multiline fixture decoding

The six negative fixtures above were added with the decoder implementation. The
official [1C:Enterprise 8.3.27 Developer Guide](https://1c-dn.com/download-trial/files/guides/developer_guide.pdf)
confirms multiline string constants, continuation lines beginning with `|`, and
doubled-quote decoding. The official [line-wrapping standard](https://kb.1ci.com/1C_Enterprise_Platform/Guides/Developer_Guides/1C_Enterprise_Development_Standards/Code_conventions/Using_1C_Enterprise_language_structures/Line_wrapping/)
shows indented continuation markers. Controlled execution on the officially
distributed 1C:Enterprise `8.3.27.2214` platform establishes the missing runtime
contract: each physical fragment boundary contributes LF, indentation before
`|` and the marker itself contribute nothing, spaces after `|` are preserved,
each empty continuation contributes one LF, doubled quotes collapse to one
quote, and LF and CRLF source-import inputs produce identical values. The full
probe source, numeric output, UTF-8 bytes, repeatability evidence, and Accepted
private source-map model are recorded in
`docs/architecture/query-language-parser-investigation.md`.

Each fixture is raw decoded query-language text. BSL opening and closing quotes,
continuation indentation, and `|` markers are omitted; each physical fragment
boundary is represented by LF; doubled BSL quotes, if present, are collapsed.
The terminal LF follows the repository fixture convention and remains ordinary
parser whitespace. The manifest distinguishes a complete statement or program
from a deliberately bounded contiguous statement excerpt.

## Deliberate evidence gaps

No raw fixture currently represents scalar parameters outside source positions,
query comments, keyword-like query string literals, or dynamically replaced
multiline text. Those cases retain their documented evidence status and do not
authorize broader expression, comment, or string grammar. Dynamic replacement
remains outside the static extractor boundary.

Malformed static query text is Accepted by ADR-0021 as a required diagnostic
case, but no repository-owned malformed raw input was established. Its fixture
therefore remains an evidence gap rather than guessed syntax. The Russian
metadata-type spelling `РегистрСведений` is repository-backed outside query
text, but its query-language form and fixture remain Unknown. Case-variant
fixtures also remain Unknown pending repository evidence.

See `docs/architecture/query-language-parser-investigation.md` for the complete
Confirmed, Accepted, and Unknown classification, source-location blocker, and
diagnostic taxonomy.
