# Query-Language Parser Investigation

## Status and scope

This investigation defines and maintains the minimum evidence-backed 1C
query-language and query-source resolution contract for the first `Reads` slice
accepted by ADR-0021. It does not define a complete 1C query grammar and does not
change BSL extraction, graph construction, metadata resolution, or edge emission.

Evidence labels used below are:

- **Confirmed by real repository source**: visible in a verified range under
  `OneAgent_EDTproject/src`;
- **Confirmed by an existing repository test**: asserted by committed Rust test
  code;
- **Confirmed by the committed prerequisite implementation**: represented by
  committed query-language parser or private EDT query-source resolver code and
  tests;
- **Accepted architecture decision**: deterministic OneAgent behavior selected
  where repository or platform evidence does not define an implementable rule;
- **Accepted by ADR-0021 but not yet represented by source evidence**: required
  policy without a repository-owned raw syntax example;
- **Confirmed by external specification**: stated by an official 1C source but
  not independently established by repository code or tests;
- **Confirmed by controlled official platform execution**: observed in two
  byte-identical executions of an isolated probe on an officially distributed
  1C:Enterprise platform build;
- **Unknown**: repository evidence is insufficient to choose a contract.

## Current `BslQuery` boundary

`crates/bsl/src/queries.rs:7-16` defines `BslQuery` as an existing Query identity,
owner, local binding, static text, and one-based BSL declaration line.
`BslQuery::text()` returns the stored text (`crates/bsl/src/queries.rs:63-67`).
The extractor accepts a complete static string in a supported constructor or
`.Text` assignment. Its decoder handles the existing one-line form and the
confirmed pipe-style multiline form without changing the public `BslQuery`
constructor or identity.

This is BSL Query extraction, not 1C query-language parsing. The extractor does
not tokenize `SELECT`, classify statements, discover sources, build an AST, or
validate query-language syntax. Dynamic or ambiguous BSL assignments are omitted
before a Query node exists, as shown by `crates/bsl/src/queries.rs:518-552` and
the production-path test at `adapters/edt/src/lib.rs:3561-3584`.

The production graph path reads modules and invokes `LineBslQueryExtractor`
(`adapters/edt/src/bsl_graph.rs:140-167`), then parses accepted query text and
emits resolved Reads only for the current fixture-backed source forms. The
integration tests in `adapters/edt/tests/reads.rs` prove stable Query identity,
ownership, Reads emission, and the absence of companion `Writes`, `References`,
or query-derived `DependsOn` edges. Query-language parsing attaches analysis to
the existing Query identity; it does not replace or split the Query node.

### Multiline BSL string implementation boundary

Real query programs commonly use multiline BSL strings whose source lines begin
with `|`. `LineBslQueryExtractor` now implements the byte-affecting rules
established by the official syntax sources below plus controlled execution on
1C:Enterprise 8.3.27.2214. Focused tests cover constructor and `.Text` forms,
LF/CRLF equivalence, empty continuations, doubled quotes, terminators,
conservative rejection, deterministic identity, and the private decoded-to-BSL
location model.

### Official multiline-string evidence

External specification evidence is deliberately separated from repository
evidence:

- the [1C:Enterprise 8.3.27 Developer Guide](https://1c-dn.com/download-trial/files/guides/developer_guide.pdf)
  defines string literals as quoted Unicode strings, requires two consecutive
  quotes to represent one quote, and describes a multiline form in which every
  continuation line starts with `|`;
- the official [1C:Enterprise Development Standards: Line wrapping](https://kb.1ci.com/1C_Enterprise_Platform/Guides/Developer_Guides/1C_Enterprise_Development_Standards/Code_conventions/Using_1C_Enterprise_language_structures/Line_wrapping/)
  prescribes `|` for wrapped string constants and shows indentation before the
  marker plus representative query-text continuation payloads.

These sources confirm syntax, not a byte-for-byte serialization algorithm for
the runtime string. In particular, Developer Guide pages 165-166 describe
runtime strings as UTF-16 values, define doubled quotes, and show the pipe-style
multiline form, but do not state marker contribution, indentation removal,
newline code units, source-ending normalization, or empty-line value. The line-
wrapping standard is a style rule with representative query text; it does not
define runtime bytes. Controlled platform execution supplies those missing
runtime facts. The decoded-to-source coordinate model remains an explicitly
Accepted OneAgent architecture decision rather than a platform claim.

### Controlled platform conformance probe

The probe used only an officially distributed local platform installation and a
fresh temporary file infobase. No repository or user infobase was executed.

| Property | Recorded value |
|---|---|
| Platform | 1C:Enterprise `8.3.27.2214`; `/opt/1cv8/8.3.27.2214/1cv8`; SHA-256 `20b74c2a82d1858db6cfb42f3873e19e426c1633a0515313f4208a02d022ce15` |
| Configuration tool | `/opt/1cv8/8.3.27.2214/ibcmd`; reported version `8.3.27.2214`; SHA-256 `1a35bba7883a7fdf9a8678392e4b1520df452995ce005ff225f94594e70cfe25` |
| Operating system | macOS `26.5` build `25F71`, `arm64` |
| Temporary root | `/private/tmp/oneagent-bsl-conformance.qDLKHp`; retained after the investigation |
| LF source | UTF-8 without BOM; 1,207 bytes; 36 LF and no CRLF; SHA-256 `22180b93a385ed7e53966c467a3c633ae3a12451b626f89a299a8e36361597d3` |
| CRLF source | UTF-8 without BOM; 1,243 bytes; 36 CRLF and no bare LF; SHA-256 `664c24b0f5c303e490f9fa81a09088c977a7c2b24ca9844e58c329595e47e19b` |

The exact logical probe source follows. The `04_tabs_before_marker` continuation
line contains two U+0009 characters before `|`; the
`03_spaces_before_marker` line contains four U+0020 characters before `|`.
The complete file was materialized once with LF and once with CRLF endings.

```bsl
Procedure OnStart()
	OutputDirectory = "/private/tmp/oneagent-bsl-conformance.qDLKHp/output";
	CreateDirectory(OutputDirectory);

	WriteProbe(OutputDirectory, "01_opening_payload", "OPEN
|NEXT");
	WriteProbe(OutputDirectory, "02_marker_no_indent", "A
|B");
	WriteProbe(OutputDirectory, "03_spaces_before_marker", "A
    |B");
	WriteProbe(OutputDirectory, "04_tabs_before_marker", "A
		|B");
	WriteProbe(OutputDirectory, "05_spaces_after_marker", "A
|  B");
	WriteProbe(OutputDirectory, "06_empty_continuation", "A
|
|B");
	WriteProbe(OutputDirectory, "07_consecutive_empty_continuations", "A
|
|
|B");
	WriteProbe(OutputDirectory, "08_doubled_quotes", "A""Б");
	WriteProbe(OutputDirectory, "09_closing_quote_terminator", "A
|Б");
	WriteProbe(OutputDirectory, "10_ascii_cyrillic", "ASCII
|Кириллица");
	WriteProbe(OutputDirectory, "11_writer_explicit_lf", "X" + Char(10) + "Y");
	WriteProbe(OutputDirectory, "12_writer_explicit_crlf", "X" + Char(13) + Char(10) + "Y");

	Exit();
EndProcedure

Procedure WriteProbe(OutputDirectory, ProbeName, Value)
	BinaryData = GetBinaryDataFromString(Value, TextEncoding.UTF8, False);
	BinaryData.Write(OutputDirectory + "/" + ProbeName + ".bin");
EndProcedure
```

The reproducible command sequence was:

```bash
/opt/1cv8/8.3.27.2214/ibcmd infobase create --data=/private/tmp/oneagent-bsl-conformance.qDLKHp/server-data --database-path=/private/tmp/oneagent-bsl-conformance.qDLKHp/infobase
/opt/1cv8/8.3.27.2214/ibcmd infobase config import --data=/private/tmp/oneagent-bsl-conformance.qDLKHp/server-data --database-path=/private/tmp/oneagent-bsl-conformance.qDLKHp/infobase /private/tmp/oneagent-bsl-conformance.qDLKHp/export
/opt/1cv8/8.3.27.2214/ibcmd infobase config apply --data=/private/tmp/oneagent-bsl-conformance.qDLKHp/server-data --database-path=/private/tmp/oneagent-bsl-conformance.qDLKHp/infobase
/opt/1cv8/8.3.27.2214/1cv8 ENTERPRISE /F /private/tmp/oneagent-bsl-conformance.qDLKHp/infobase /DisableStartupDialogs /DisableStartupMessages /Out /private/tmp/oneagent-bsl-conformance.qDLKHp/run-binary-<ending>-<run>.log
```

The module imported and the database configuration applied successfully for
both source-ending forms. Each client run exited with status 0. Each `/Out` log
contained only the UTF-8 BOM `EF BB BF`, with no diagnostic text. The probe used
`GetBinaryDataFromString(..., TextEncoding.UTF8, False)` and `BinaryData.Write`,
so each result file contains exactly the listed UTF-8 bytes without a BOM. The
explicit LF and CRLF controls prove that this serialization path preserves both
sequences unchanged. An earlier `TextWriter.Write` instrumentation attempt was
rejected because its controls proved that it normalizes LF to CRLF; those
rejected outputs remain in the temporary evidence directory and are not used
for any decoding conclusion.

| Probe | UTF-16 code units / Unicode code points | Length | First / last | UTF-8 payload bytes |
|---|---|---:|---|---|
| `01_opening_payload` | `004F 0050 0045 004E 000A 004E 0045 0058 0054` | 9 | `U+004F` / `U+0054` | `4f50454e0a4e455854` |
| `02_marker_no_indent` | `0041 000A 0042` | 3 | `U+0041` / `U+0042` | `410a42` |
| `03_spaces_before_marker` | `0041 000A 0042` | 3 | `U+0041` / `U+0042` | `410a42` |
| `04_tabs_before_marker` | `0041 000A 0042` | 3 | `U+0041` / `U+0042` | `410a42` |
| `05_spaces_after_marker` | `0041 000A 0020 0020 0042` | 5 | `U+0041` / `U+0042` | `410a202042` |
| `06_empty_continuation` | `0041 000A 000A 0042` | 4 | `U+0041` / `U+0042` | `410a0a42` |
| `07_consecutive_empty_continuations` | `0041 000A 000A 000A 0042` | 5 | `U+0041` / `U+0042` | `410a0a0a42` |
| `08_doubled_quotes` | `0041 0022 0411` | 3 | `U+0041` / `U+0411` | `4122d091` |
| `09_closing_quote_terminator` | `0041 000A 0411` | 3 | `U+0041` / `U+0411` | `410ad091` |
| `10_ascii_cyrillic` | `0041 0053 0043 0049 0049 000A 041A 0438 0440 0438 043B 043B 0438 0446 0430` | 15 | `U+0041` / `U+0430` | `41534349490ad09ad0b8d180d0b8d0bbd0bbd0b8d186d0b0` |
| `11_writer_explicit_lf` | `0058 000A 0059` | 3 | `U+0058` / `U+0059` | `580a59` |
| `12_writer_explicit_crlf` | `0058 000D 000A 0059` | 4 | `U+0058` / `U+0059` | `580d0a59` |

All characters are in the BMP, so the reported UTF-16 code-unit count equals
the Unicode scalar-value count. The two LF executions were byte-identical, the
two CRLF executions were byte-identical, and LF output was byte-identical to
CRLF output for all twelve files (`diff -rq` produced no output in each
comparison). The platform's XML import/export path normalizes the stored module
to CRLF, but the independently recorded LF and CRLF inputs establish that the
official source-import path does not preserve their physical newline spelling
in the runtime value. Both inputs produce one U+000A per fragment boundary.

### Inspected multiline declarations

| Source declaration | Complete enclosing range and use | Classification | Fixture suitability |
|---|---|---|---|
| `TextProductsAndNamedProducts` | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:154-181`; returned at line 156, selected at line 103, assigned to `Query.Text` at line 106, then passed to a query-mutating helper at lines 108-109 | **Confirmed** static multiline return value, indirectly consumed and subsequently modified; literal decoding is deterministic | Unsuitable for full-builder raw evidence because the executed value is subsequently modified |
| `QueryCalendarBatch` | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:183-322`; direct multiline `Query.Text` assignment at lines 187-307, followed by `StrReplace(Query.Text, ...)` at lines 313-317 | **Confirmed** static multiline assignment followed by reassignment/replacement; literal decoding is deterministic | Unsuitable because the final query text is dynamic |
| `ReportDataQueryText` | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:169-359`; one returned multiline literal at lines 173-357 and direct call assignment to `Query.Text` at line 128 | **Confirmed** static multiline return value, indirectly consumed; literal decoding is deterministic | Structurally useful for `UNION ALL`, batches, temporary tables, nested queries, virtual tables, and joins, but not consumable by the current `LineBslQueryExtractor` |
| Conditional attachment query | `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:133-248`; representative branch literal at lines 135-164, alternate assignments through line 243, and `StrReplace` assignment to `Query.Text` at line 248 | **Confirmed** conditional static templates with a parameter-source placeholder, then dynamically replaced; each literal is deterministically decodable | Unsuitable because no single literal is the final query program |
| Catalog query batch template | `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:2234-2255`; multiline template at lines 2235-2242, per-catalog replacement at lines 2244-2251, and runtime `StrConcat` with inserted `Chars.LF` and `UNION ALL` at line 2255 | **Confirmed** replaced, concatenated, dynamically assembled query text; the static template is deterministically decodable | Unsuitable because the final batch is not one source literal |
| `MarkedObjectsDeletionControl` query template | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:325-352`; multiline template at lines 337-348 and two replacements at lines 349-352 | **Confirmed** static multiline template followed by replacement and divergent values; the static template is deterministically decodable | Unsuitable because the parser input is dynamically assembled |
| `PrepareProducts` query constructor | `OneAgent_EDTproject/src/CommonModules/wms_mobile_ProductsPicking/Module.bsl:397-479`; the complete constructor argument is one multiline literal at lines 399-479 | **Confirmed** static multiline literal passed directly to `New Query`, including empty continuation lines and doubled quotes; decoding is deterministic | Preferred next extractor and fixture candidate |

All five repository files use LF source line endings in the committed snapshot.
That is a source-file fact only; it does not prove which newline code units the
1C runtime places in the resulting string.

### Multiline decoding evidence matrix

| Decoding rule | Repository evidence | External specification evidence | Classification | OneAgent consequence |
|---|---|---|---|---|
| Opening-line payload after the first quote | One-line decoder and tests preserve characters between delimiters; every inspected multiline declaration has payload after the opening quote | String literals are characters enclosed in quotes | **Confirmed** for ordinary literal payload | Preserve the opening fragment verbatim except for confirmed quote decoding |
| Continuation-line `|` is required syntax | Every inspected multiline declaration uses it | Developer Guide requires each continuation line to start with `|`; development standard prescribes it | **Confirmed** | A later extractor may recognize only this evidenced continuation form |
| Runtime contribution of `|` | No decoder exists yet | Official prose is incomplete; platform probes 02-07 produce no U+007C | **Confirmed** by controlled official platform execution | Discard the syntactic marker |
| Indentation before `|` | Repository examples use different tab/space depths | Platform probes 03 and 04 are byte-identical to unindented probe 02 | **Confirmed** by controlled official platform execution | Discard all U+0020 and U+0009 before the marker; preserve payload after it |
| Spaces after `|` | Repository queries use payload spacing after the marker | Platform probe 05 preserves both U+0020 characters | **Confirmed** by controlled official platform execution | Copy every payload byte after the marker, subject only to doubled-quote decoding |
| Newline insertion between fragments | Source declarations span physical lines and contain empty continuation lines | Every platform fragment boundary produces U+000A | **Confirmed** by controlled official platform execution | Insert UTF-8 byte `0A` before each continuation payload |
| LF versus CRLF | Inspected repository files contain LF and no CRLF | LF and CRLF input modules produce byte-identical output in two runs each | **Confirmed** for the official XML source-import path | Normalize either physical source ending to decoded LF |
| Doubled quote `""` | `PrepareProducts` uses doubled quotes at lines 460-463; the current one-line decoder converts `""` to `"` | Developer Guide explicitly defines two quotes as one quote character | **Confirmed** | Decode each evidenced doubled-quote pair to one quote character |
| Closing quote and statement terminator | Current one-line decoder excludes enclosing quotes; statement parsing removes a BSL `;` outside the literal | Official literal syntax uses enclosing quotes; examples place the BSL terminator after the closing quote | **Confirmed** | Exclude the closing quote and the BSL statement terminator from query text |
| Empty continuation line | Empty `|` lines occur in the inspected declarations | Probe 06 adds one LF and probe 07 adds two consecutive LF characters | **Confirmed** by controlled official platform execution | Each empty continuation contributes exactly one inserted LF and zero payload bytes |
| UTF-8 byte locations in decoded query text | `QueryTextRange` is zero-based, half-open, and slices unchanged Rust UTF-8 input | Platform strings are specified as UTF-16, not Rust UTF-8 | **Accepted** as the parser-local OneAgent coordinate system after decoding | Keep parser ranges in decoded UTF-8 bytes; this does not solve BSL mapping |
| Mapping decoded ranges to BSL lines and columns | `BslQuery` stores only one declaration line; no column, segment map, or multiline model exists | Platform semantics define values, not diagnostic projection | **Accepted** private OneAgent segment model | Implement the mapping contract below without changing the public `BslQuery` API |

Every byte-affecting transformation required for the repository-used pipe form
is Confirmed. The accepted parser-local UTF-8 range contract remains unchanged
and starts after the deterministic BSL decoder produces the raw query `str`.

### Accepted private BSL-to-query source map

The extractor implementation must retain an internal ordered segment map. This
is an Accepted architecture decision; the platform does not define OneAgent
diagnostic coordinates.

- `Copied` maps a zero-based half-open decoded UTF-8 byte range to the exact
  zero-based half-open UTF-8 source-byte range containing ordinary payload.
- `CollapsedQuote` maps the one decoded U+0022 byte to the half-open source
  range containing both consecutive BSL quote bytes.
- `InsertedLf` maps the decoded one-byte `0A` range to a physical boundary,
  not to fictional copied bytes. The boundary retains the previous physical
  line-ending range and the next line's indentation-plus-marker range.
- A physical-line table retains the one-based BSL line number, raw content and
  ending ranges, marker offset, and payload-start offset. One-based BSL columns
  are derived by counting Unicode scalar values from the physical line start;
  absolute source storage and all parser ranges remain UTF-8 byte based.
- A diagnostic range touching an inserted LF projects to the continuation
  marker's one-based line and column and may carry the complete boundary span.
  A range touching copied or collapsed bytes projects through the corresponding
  source range. Cross-segment ranges use the earliest projected start and latest
  projected end.

This model distinguishes copied payload, two-to-one quote decoding, inserted
bytes, and physical line boundaries without changing `QueryTextRange`, public
`BslQuery`, Query identity, or graph provenance APIs.

## Evidence inventory

| Finding | Evidence | Status | First-slice consequence |
|---|---|---|---|
| Direct English Catalog source | `OneAgent_EDTproject/src/Catalogs/AdditionalReportsAndDataProcessors/Forms/QuickAccessToAdditionalReportsAndDataProcessors/Module.bsl:140-144` contains one static line with `FROM Catalog.Users`; `crates/bsl/src/queries.rs:455-474` and `adapters/edt/src/lib.rs:3461-3477` use `Catalog.Products` | Confirmed by real repository source and an existing repository test | `Catalog.<Name>` is repository-backed |
| Direct English Information Register source and alias | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:764-771` contains `FROM InformationRegister.ObjectsToDelete AS Tab` | Confirmed by real repository source | `InformationRegister.<Name>` and an optional `AS` alias are repository-backed |
| Query source and EDT technical-name pairs | `Catalog.Users` matches `<name>Users</name>` in `OneAgent_EDTproject/src/Catalogs/Users/Users.mdo:10`; `InformationRegister.ObjectsToDelete` matches `<name>ObjectsToDelete</name>` in `OneAgent_EDTproject/src/InformationRegisters/ObjectsToDelete/ObjectsToDelete.mdo:12` | Confirmed by real repository source | Resolution uses the EDT technical `<name>`, not a synonym or directory spelling |
| Case differences in representative source pairs | A case-insensitive scan of direct `Catalog`, `InformationRegister`, `Справочник`, and `РегистрСведений` source forms found no production source whose local name differed by case from its corresponding EDT technical name | Confirmed negative repository search, not a platform semantic claim | Repository source neither proves nor disproves case-insensitive metadata-name lookup |
| Scalar parameter outside a source position | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:337-352` contains `&UnlockTime` in `WHERE` and a direct Information Register source | Confirmed by real repository source | ADR-0021 permits it only after the entire expression and source set are parsed; no raw fixture is available because the evidence is multiline |
| English keywords | The preceding ranges show `SELECT`, `FROM`, `AS`, `WHERE`, `TOP`, `NOT`, and `AND` | Confirmed by real repository source | Only observed spellings are evidence-backed; this is not a complete keyword list |
| Russian keywords and Catalog namespace | `crates/bsl/src/queries.rs:477-496` contains `ВЫБРАТЬ Ссылка ИЗ Справочник.Номенклатура` | Confirmed by an existing repository test | `ВЫБРАТЬ`, `ИЗ`, and `Справочник` are repository-backed parser inputs |
| Russian Information Register namespace | `OneAgent_EDTproject/src/CommonModules/AccessManagementInternal/Module.bsl:40055-40063` maps the metadata-type spelling `РегистрСведений` to `InformationRegister`, but no representative occurrence was established inside query-language text | Confirmed by real repository source as a metadata-type spelling; Unknown as query-language syntax | Do not infer `РегистрСведений.<Имя>` parser support from a non-query mapping or English query evidence |
| Query comments | `OneAgent_EDTproject/src/CommonModules/DataExchangeServer/Module.bsl:7895-7923` contains `//` query comments and statement separators; `OneAgent_EDTproject/src/DataProcessors/SaleWizard/Forms/Form/Module.bsl:589-605` uses a `//FilterByCustomer` marker inside dynamically replaced text | Confirmed by real repository source for comment-shaped text; keyword-token payload coverage is Unknown | The lexer must shield comment content from keyword/source recognition; raw decoding and exact comment grammar remain unproven |
| Query string literals | `OneAgent_EDTproject/src/CommonModules/wms_mobile_ProductsPicking/Module.bsl:458-466` contains query values whose BSL representation includes `""CatalogRef.Products""` and `""CatalogRef.NamedProducts""` | Confirmed by real repository source for metadata-like literal content; exact `SELECT`/`FROM`-like literal content is Unknown | String contents must not become source occurrences; decoded quoting rules remain unproven |
| Statement delimiters and batches | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:173-192` and `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:156-179` contain `;` followed by another `SELECT` | Confirmed by real repository source | More than one statement is unsupported for the first slice |
| `UNION` | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:173-189` contains `UNION ALL`; `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:167-179` contains `UNION` | Confirmed by real repository source | Unsupported structure; the whole Query has no complete accepted source set |
| `JOIN` | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:167-179` contains `INNER JOIN`; `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:135-164` contains `LEFT JOIN` | Confirmed by real repository source | Unsupported structure, regardless of join kind |
| Nested query | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:221-233` contains a parenthesized `SELECT` in a filter | Confirmed by real repository source | Unsupported nested source scope |
| Temporary-table declaration, read, and batch | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:156-179` declares `INTO ttProducts`, then reads `ttProducts` in another statement | Confirmed by real repository source | Unsupported temporary-table structure and multiple statements |
| Virtual table | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:258-273` invokes `AccumulationRegister.QuantitativeAccounting.Balance(...)`; `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:350-358` invokes `.Turnovers(...)` | Confirmed by real repository source | Unsupported source; do not degrade it to a read of the base register |
| Parameter-supplied source | `OneAgent_EDTproject/src/DataProcessors/ExportImportEnterpriseData/ObjectModule.bsl:871-877` contains the one-line template `FROM &MetadataTableName AS MDTableAlias`; `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:135-164` contains `FROM &TableAttachedFiles` | Confirmed by real repository source | Unsupported external or parameter source |
| Dynamically replaced source text | `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:2235-2247` replaces `&CatalogName`; `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:306-317` replaces part of `Query.Text`; `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:337-352` replaces conditions | Confirmed by real repository source | This is a BSL extraction/evidence failure when complete static text is unavailable, not query-language malformed syntax |
| Malformed static query text | No committed raw malformed query-language example was found | Accepted by ADR-0021 but not yet represented by source evidence | A syntax diagnostic is required, but exact malformed examples and recovery behavior remain a fixture blocker |
| Incomplete BSL Query declaration | `crates/bsl/src/queries.rs:518-552` covers dynamic, ambiguous, module-scope, and missing-static-text patterns | Confirmed by an existing repository test | These patterns produce no parser input and must not be reported as query-language syntax failures |

## Authoritative and implementation references

The official 1C:Enterprise Developer Guide states that module-language names
and keywords are case-insensitive, while the official query-language overview
directs readers to the platform's built-in query help. Neither inspected source
states an exact Unicode comparison algorithm for query metadata identifiers.
Module-language behavior is therefore adjacent evidence only and is not promoted
to a query-language fact:

- [1C:Enterprise Developer Guide, module format](https://kb.1ci.com/1C_Enterprise_Platform/Guides/Developer_Guides/1C_Enterprise_8.3.23_Developer_Guide/Chapter_4._1C_Enterprise_language/4.2._Format_of_module_source_text/4.2.4._Module_format/?language=en);
- [1C:Enterprise query-language overview](https://1c-dn.com/library/tutorials/practical_developer_guide_query_language/).

Rust defines [`str::to_lowercase`](https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase)
as Unicode lowercase conversion that can expand characters, handles contextual
Greek sigma, and does not apply locale-specific Turkish or Azeri casing. This is
the authoritative implementation behavior selected below; it is not evidence
that the 1C platform uses the same Unicode algorithm.

## Minimum lexical observations

The fixture corpus proves only the following minimum observations:

1. UTF-8 input is required because accepted evidence includes Russian text.
2. Whitespace separates tokens in the observed examples.
3. `.` separates a namespace and local metadata name.
4. `&` introduces an observed parameter token.
5. English and Russian keyword spellings must be recognized as language tokens,
   not searched as substrings.
6. `//` occurs as a query-comment prefix in real source, and quoted query values
   can contain metadata-like text. Their full lexical rules remain Unknown until
   raw decoded fixtures are established.
7. `;` occurs between statements in real query batches. Whether one optional
   trailing delimiter is accepted for a single statement is Unknown.

The repository does not prove escape rules, identifier quoting, all whitespace
forms, numeric/date literal forms, comment termination at raw query EOF, or a
complete bilingual keyword equivalence table.

## Minimum structural boundary

The later parser must consume the complete raw query program through end of
input. Successful first-slice classification requires all of the following:

- exactly one statement;
- one top-level English or evidence-backed Russian `SELECT` form;
- exactly one top-level source clause;
- one direct qualified persistent source in the allowlist;
- an optional source alias;
- no second source branch or source-producing construct;
- complete consumption of projections, filters, ordering, and other accepted
  tails, rather than treating an unparsed tail as harmless;
- a positive `complete_source_set` result only after the preceding conditions
  are proved.

The minimum typed parsed result required by ADR-0021 is a program classification,
deterministically ordered source occurrences, and for each occurrence its raw
spelling, normalized category, namespace, local metadata name, and query-text
location. This investigation deliberately does not prescribe a Rust AST or
public API.

### Accepted source categories

| Raw source form | Parsed category | Evidence readiness |
|---|---|---|
| `Catalog.<Name>` | Direct persistent Catalog source | Ready in English |
| `InformationRegister.<Name>` | Direct persistent Information Register source | Ready in English |
| `Справочник.<Имя>` | Direct persistent Catalog source | Test-backed, without real-source corpus |
| `РегистрСведений.<Имя>` | Direct persistent Information Register source | Namespace spelling is repository-backed outside query text; query-language corpus is Unknown and not ready |

Raw spelling must be preserved separately from normalized category. A source
alias is not a target name. Aliases can be included without weakening proof
because a one-line real Information Register query proves the syntax and still
has exactly one direct source. Scalar parameters outside source positions are
contractually admissible and present in real source, but accepting them in the
first implementation is not ready until a raw fixture and sufficient expression
grammar prove full consumption. A parser may conservatively reject such a query;
it must not accept it by ignoring the parameter-bearing tail.

### All-or-nothing unsupported categories

Any of the following prevents complete-source proof for the entire Query:

- any `JOIN`;
- `UNION` or `UNION ALL`;
- a nested query;
- more than one statement or any batch;
- a temporary-table declaration or source;
- a register virtual table or source invocation;
- an external or parameter-supplied data source;
- another persistent metadata namespace outside the allowlist;
- malformed or unconsumed query text.

Dynamic, replaced, reassigned, ambiguous, or incomplete BSL text is a preceding
boundary: it normally prevents an existing static Query parser input. It is not
the same outcome as malformed static query text attached to an existing Query
node. Neither outcome authorizes a partial source set or a `Reads` edge.

## Identifier evidence and accepted lookup contract

Evidence establishes dotted qualified names and aliases with the exact spellings
shown in the inventory. It does not establish quoted identifiers, non-ASCII
metadata identifiers in real query source, or identifier character rules.

The BSL extractor lowercases BSL constructor/property keywords internally, and
the local and cross-module call resolvers use `str::to_lowercase`. Those are
repository precedents, not proof of query-language case semantics. The inspected
query-language examples use uppercase keywords and consistently cased namespace
and metadata names. No inspected authoritative 1C source specifies Unicode
normalization or case folding for query metadata identifiers.

The first-slice resolver adopts the following **Accepted architecture decision**
to make lookup deterministic without claiming complete platform equivalence:

```rust
fn query_source_lookup_key(value: &str) -> String {
    value.to_lowercase()
}
```

The contract is exact:

1. Local metadata-name matching is case-insensitive under equality of this
   lookup key. Both `QuerySourceOccurrence::local_name()` and each graph node's
   complete `EntityName` are transformed independently.
2. Conversion is Rust Unicode lowercase over the complete `str`, not ASCII-only
   folding and not Unicode Default Case Folding. It is independent of process,
   user, operating-system, database, and 1C session locale.
3. The conversion may expand one Unicode scalar value into multiple scalar
   values. The complete expanded sequence is part of the key; for example,
   `İ` lowercases to `i` followed by U+0307 COMBINING DOT ABOVE. No character is
   truncated and no combining mark is removed.
4. No NFC, NFD, NFKC, or NFKD normalization is applied before or after
   lowercasing. Canonically or compatibility-equivalent spellings remain
   different keys unless `str::to_lowercase()` alone makes them byte-identical.
5. Already-lowercase English and Russian names remain unchanged. English
   `A`-`Z`, Russian `А`-`Я`, and `Ё` use their Rust Unicode lowercase mappings;
   uncased non-ASCII scalar values remain unchanged. The same algorithm applies
   to every accepted identifier rather than branching by script.
6. The raw namespace, local name, and qualified spelling remain unchanged in
   `QuerySourceOccurrence` for diagnostics and later provenance. A lookup key is
   derived data and must not replace, rewrite, or become part of Query identity.

Namespace handling is deliberately separate. The parser remains the sole owner
of the explicit mapping from raw namespace spelling to `QuerySourceCategory`.
The current minimum parser accepts only the exact allowlist spellings represented
by its fixtures; therefore namespace matching is case-sensitive at this parser
boundary. The metadata resolver consumes `QuerySourceCategory`, never
reclassifies `namespace()`, never infers an additional namespace, and never uses
the raw namespace as a graph lookup key. A later evidence-backed parser expansion
may compare namespace tokens case-insensitively, but it must normalize both the
candidate and explicit allowlist entries inside the parser and must not move the
allowlist into metadata resolution.

English and Russian case variants are contract tests for the resolver rather
than claimed production-source facts. The existing accepted queries can resolve
against graph technical names such as `products` and `номенклатура`; compatible
nodes named `Products` and `PRODUCTS`, or `Номенклатура` and `НОМЕНКЛАТУРА`, must
exercise collision behavior. No new raw parser fixture is required merely to
test graph-name normalization.

### Candidate index and exact-kind filtering

The resolver builds a private immutable index from one completed graph snapshot:

```text
lookup key -> deterministically ordered graph-node candidates
```

Candidate IDs are stored in `BTreeSet<EntityId>` or an equivalent total ordering.
The index retains each candidate's original name and `NodeKind`; it does not
insert normalized names into `SemanticGraph`. The expected kind comes only from
the parsed category:

| Parsed category | Exact compatible target kind |
|---|---|
| `QuerySourceCategory::Catalog` | `NodeKind::Metadata(MetadataKind::Catalog)` |
| `QuerySourceCategory::InformationRegister` | `NodeKind::Metadata(MetadataKind::InformationRegister)` |

No other metadata kind, metadata member, flat semantic node, `Unknown`, external,
placeholder, synonym, display name, historical name, or lower-confidence target
is compatible.

Exact kind partitions candidates before cardinality is evaluated. This preserves
the existing `SemanticResolutionIndex::resolve_name_of_kind` precedent:

- exactly one compatible candidate succeeds even when one or more incompatible
  nodes have the same lookup key;
- two or more compatible candidates produce a typed ambiguous outcome containing
  only compatible candidate IDs, sorted by `EntityId`;
- no compatible candidate and one or more incompatible candidates produce a
  typed incompatible-kind outcome; if that outcome carries candidate details,
  they must also be sorted by `EntityId`;
- collisions exclusively among incompatible kinds do not become compatible
  ambiguity because the parsed namespace has already fixed the required kind.

Thus a normalized-name collision across compatible and incompatible kinds is
deterministic: a unique exact-kind candidate wins; compatible multiplicity wins
the ambiguous precedence; and incompatible candidates matter only when no exact
kind is present.

### Failure precedence and partial workspace

Resolution is all-or-nothing at the parsed-program boundary. It starts only when
`QueryLanguageParseResult::is_source_set_complete()` is true, a program exists,
there are no parser diagnostics, and every source occurrence belongs to the
accepted category set. It starts only after the EDT builder has completed
insertion of all top-level metadata nodes for the supplied workspace snapshot.
No resolver diagnostic is produced for rejected or partial parser output.

For each accepted occurrence, outcomes use this precedence:

1. two or more exact-kind candidates: ambiguous;
2. exactly one exact-kind candidate: resolved;
3. no exact-kind candidate but at least one differently typed candidate:
   incompatible kind;
4. no candidate of any kind and an explicit partial-workspace signal:
   partial-workspace absence;
5. no candidate of any kind and an explicit complete-workspace signal: missing
   target.

The private implementation must represent these states with an equivalent typed
shape; the names below are canonical for the implementation task unless nearby
code requires a strictly private naming adjustment:

```rust
enum QuerySourceResolutionOutcome {
    Resolved { target_id: EntityId },
    MissingTarget,
    AmbiguousTarget { candidates: Vec<EntityId> },
    IncompatibleTargetKind { candidates: Vec<EntityId> },
    PartialWorkspaceTargetAbsent,
}

enum WorkspaceResolutionScope {
    Complete,
    Partial,
}
```

Both candidate vectors are sorted and deduplicated by `EntityId`. Resolved maps
to `ResolutionState::Resolved`, ambiguous to `ResolutionState::Ambiguous`,
partial-workspace absence to `ResolutionState::Partial`, and missing or
incompatible to `ResolutionState::Unresolved` when these outcomes are later
adapted into graph diagnostics.

Partial-workspace absence is not inferred from a missing metadata directory, an
empty candidate set, or an unresolved name. The private resolver input must carry
an explicit complete-versus-partial workspace scope supplied by the caller. A
successful full-project EDT scan supplies complete scope; a future partial
importer must supply partial scope. Incompatible or ambiguous in-graph evidence
takes precedence over workspace completeness. All four failures are typed, emit
no target or edge, and retain the raw source spelling and Query identity.

### Resolver ownership and dependency direction

The implementation belongs in a private `oneagent-edt` module, expected as
`adapters/edt/src/query_source_resolution.rs`, invoked from the existing EDT BSL
graph integration after top-level metadata collection and accepted query parsing.
This is the narrowest production layer that can consume both
`oneagent_bsl::QuerySourceOccurrence` and `oneagent_graph::SemanticGraph`:

- `oneagent-bsl` depends only on `oneagent-common` and cannot depend on
  `oneagent-graph` without reversing its source-analysis boundary;
- `oneagent-graph` is source-independent and must not depend on query-language
  types;
- `oneagent-edt` already depends on both crates and owns graph construction phase
  ordering and workspace-scope evidence;
- `oneagent-analysis` also depends on both crates, but it does not own the EDT
  production graph-construction phase or workspace completeness.

The first implementation uses the private query-source index described above.
It must not extend `SemanticResolutionIndex`, change the public Query API, or add
a graph resolution API while the case and partial-workspace policy remains
query-source-specific. A later task may generalize the index only after another
producer proves the same normalization and collision contract. The resolver-only
task returns typed outcomes and emits no `Reads`, `References`, `DependsOn`, or
other graph edge.

## Accepted source-location boundary

| Alternative | Repository evidence | Decision |
|---|---|---|
| One-based BSL line and column | Existing BSL models expose a one-based declaration line; the Accepted private segment and physical-line model defines projection without changing that API | Accepted for private diagnostic projection |
| Unicode scalar columns | Rust source is UTF-8; counting scalar values from each physical line start gives deterministic one-based user-facing columns | Accepted for BSL columns only |
| UTF-8 byte offsets | The current prerequisite parser defines `QueryTextRange` as zero-based, half-open UTF-8 byte offsets and tests slicing of English and Russian raw input | Accepted by the prerequisite implementation |

The accepted parser coordinate system is the decoded raw query `str` with a
zero-based inclusive `start_byte` and exclusive `end_byte`. The parser does not
normalize input, so original whitespace and CRLF bytes contribute to offsets and
token boundaries remain UTF-8 boundaries. The extractor owns projection back to
BSL through the private segment map defined above. The existing one-based
`BslQuery::line()` continues to identify the wrapper declaration; no public
location API was added.

## Diagnostic taxonomy

The later pipeline must distinguish stages and typed outcomes without requiring
these names to become Rust API variants:

| Stage | Required diagnostic category | Meaning |
|---|---|---|
| BSL extraction | Dynamic or incomplete BSL Query text | No complete static parser input or Query node exists; retain the current conservative extraction behavior |
| Query lexer/parser | Malformed query syntax | Static raw query text cannot be tokenized or parsed completely |
| Query parser | Unsupported query structure | Parsed `JOIN`, `UNION`, nested query, batch, or another structure outside the first slice |
| Query source classification | Unsupported persistent namespace | Direct persistent namespace is parsed but not allowlisted |
| Query source classification | Virtual table source | A register virtual table/invocation is parsed |
| Query source classification | Temporary table | A temporary-table declaration or source is parsed |
| Query source classification | External or parameter data source | A parameter or caller-supplied table occupies a source position |
| Metadata resolution | Missing metadata target | An allowlisted parsed source has no candidate of any kind in an explicitly complete workspace scope |
| Metadata resolution | Ambiguous metadata target | More than one compatible target remains, with deterministically ordered candidates |
| Metadata resolution | Incompatible metadata target kind | One or more names match the lookup key, but all occur at disallowed kinds |
| Metadata resolution | Partial-workspace absence | No in-graph candidate exists and the caller explicitly marks the workspace scope partial |

Every query-language diagnostic must identify the existing Query identity and a
query-text location once that blocked contract is decided. Diagnostics and
source occurrences must have deterministic ordering independent of fixture,
filesystem, map, or traversal order. Unsupported syntax must not be silently
ignored, and resolver diagnostics must not be produced when parsing has not
proved a complete accepted source set.

## Fixture manifest

The repository-owned raw corpus lives under
`crates/bsl/tests/fixtures/query_language`. Fixture order has no semantic meaning.

| Fixture | Provenance class | Origin | Expected classification | Expected diagnostic |
|---|---|---|---|---|
| `accepted_catalog_en.query` | Copied from an existing repository test | `crates/bsl/src/queries.rs:455-474` | One statement; one direct persistent Catalog source, raw name `Catalog.Products` | None |
| `accepted_catalog_ru.query` | Copied from an existing repository test | `crates/bsl/src/queries.rs:477-496` | One statement; one direct persistent Catalog source, raw name `Справочник.Номенклатура` | None |
| `accepted_information_register_en.query` | Verbatim decoded query text from a real one-line BSL string | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:764-771` | One statement; one direct persistent Information Register source, alias `Tab` | None |
| `unsupported_parameter_source_en.query` | Verbatim decoded query text from a real one-line BSL string | `OneAgent_EDTproject/src/DataProcessors/ExportImportEnterpriseData/ObjectModule.bsl:871-877` | One statement; parameter-supplied source, alias `MDTableAlias`; source set unsupported | External or parameter data source |
| `unsupported_join_en.query` | Decoded contiguous statement excerpt | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:145-150` | Multiple top-level sources through `INNER JOIN` | Unsupported query structure |
| `unsupported_union_all_en.query` | Decoded complete statement | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:173-189` | Multiple branches through `UNION ALL` | Unsupported query structure |
| `unsupported_nested_query_en.query` | Decoded complete statement | `OneAgent_EDTproject/src/Reports/AnalyticalReportByCategories/ObjectModule.bsl:145-158` | Parenthesized nested `SELECT` | Unsupported query structure |
| `unsupported_batch_en.query` | Decoded complete two-statement literal | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:134-150` | Statement delimiter followed by another `SELECT` | Unsupported query structure |
| `unsupported_temporary_table_en.query` | Decoded first-statement excerpt | `OneAgent_EDTproject/src/Reports/SalesAnalytics/ObjectModule.bsl:156-163` | `INTO ttProducts` declaration | Temporary table source |
| `unsupported_virtual_table_en.query` | Decoded complete statement | `OneAgent_EDTproject/src/Reports/TransferOfProduct/Forms/ReportForm/Module.bsl:259-273` | Register virtual-table invocation | Virtual table source |

Missing raw fixtures for scalar parameters, comments, keyword-like string
literals, and malformed syntax remain explicit evidence gaps. The negative
multiline derivatives use only confirmed decoding rules and retain exact source
provenance in this manifest and the fixture README.

## Implementation-readiness matrix

| Lexical or structural rule | Evidence status | Fixture | Expected parsed classification | Expected diagnostic | Remaining blocker |
|---|---|---|---|---|---|
| English `SELECT ... FROM Catalog.<Name>` | Confirmed by source and tests | `accepted_catalog_en.query` | Direct Catalog source | None | Full projection/tail grammar remains intentionally narrow |
| English `SELECT ... FROM InformationRegister.<Name> AS <Alias>` | Confirmed by source | `accepted_information_register_en.query` | Direct Information Register source with alias | None | None for this exact shape |
| Russian `ВЫБРАТЬ ... ИЗ Справочник.<Имя>` | Confirmed by test | `accepted_catalog_ru.query` | Direct Catalog source | None | No representative real-source corpus |
| Russian Information Register form | Namespace spelling confirmed outside query text; query-language form Unknown | None | Direct Information Register source | None | Repository-backed query-language occurrence and fixture |
| Scalar parameter outside source position | Confirmed by source; accepted by ADR-0021 | None | Still one direct persistent source if the expression parses fully | None | Raw decoded fixture and complete expression consumption |
| Parameter in source position | Confirmed by source | `unsupported_parameter_source_en.query` | Unsupported parameter source | External or parameter data source | None for this exact raw shape |
| Optional alias | Confirmed by source | `accepted_information_register_en.query` | Alias attached to one source occurrence | None | Russian alias spelling is not evidenced |
| Comment shielding | Comment-shaped text confirmed only in multiline BSL source; keyword-token payload Unknown | None | Comments contribute no source tokens | Malformed syntax only if comment lexing itself fails | Raw decoding, keyword-like fixture, and exact termination rules |
| String-literal shielding | Metadata-like literal content confirmed only in multiline BSL source; keyword-token payload Unknown | None | Literal content contributes no source tokens | Malformed syntax for unterminated literal | Raw quoting/escape contract and keyword-like fixture |
| Statement delimiter and batch detection | Structure and multiline decoding Confirmed | `unsupported_batch_en.query` | Multiple statements | Unsupported query structure | None for the evidenced delimiter-followed-by-statement shape |
| `JOIN` detection | Structure and multiline decoding Confirmed | `unsupported_join_en.query` | Multiple top-level sources | Unsupported query structure | None for evidenced join keywords; broader join grammar remains deferred |
| `UNION` detection | Structure and multiline decoding Confirmed | `unsupported_union_all_en.query` | Multiple branches | Unsupported query structure | None for `UNION` and `UNION ALL` detection |
| Nested-query detection | Structure and multiline decoding Confirmed | `unsupported_nested_query_en.query` | Nested source scope | Unsupported query structure | None for the evidenced parenthesized `SELECT` shape |
| Temporary-table detection | Structure and multiline decoding Confirmed | `unsupported_temporary_table_en.query` | Temporary declaration/source | Temporary table source | None for evidenced `INTO` declaration and unqualified source use |
| Virtual-table detection | Structure and multiline decoding Confirmed | `unsupported_virtual_table_en.query` | Virtual table source | Virtual table source | None for evidenced register third-component invocation |
| Malformed static input | Accepted by ADR-0021, not source-evidenced | None | No complete parsed program | Malformed query syntax | Evidence-backed malformed corpus and recovery policy |
| Deterministic query-text location | Parser-local range and private multiline source map are implemented | Existing fixtures plus inline parser/extractor tests | Zero-based half-open UTF-8 byte range in decoded query text, projected through typed private segments | Applicable typed diagnostic | None for the accepted private model |
| Case normalization | Accepted architecture decision | Existing accepted English and Russian queries can be paired with case-variant graph names in resolver tests | Preserved raw spelling plus locale-independent Rust Unicode lowercase key; no NFC/NFKC | Typed resolver outcome only after complete parsing | Resolver tests must cover English, Russian, expansion, no-normalization, and collisions |
| Exact-kind resolution | Implemented by the committed private EDT resolver | Committed private resolver tests | Catalog or Information Register exact-kind candidate partition | Missing, ambiguous, incompatible, or partial-workspace | None for the accepted resolver slice |

## Readiness conclusion and next task boundary

The committed prerequisite parser establishes complete-source proof, raw
spelling, typed source categories, and deterministic raw-query byte locations
for its minimum fixture-backed forms. The committed private EDT resolver applies
the accepted normalization, collision, failure-precedence, workspace-scope, and
exact-kind rules. Precise graph validation accepts only
`Query --Reads--> Metadata(Catalog | InformationRegister)` and leaves `Writes`
broadly accepted. The EDT production builder now invokes parsing and resolution
after top-level metadata collection, emits canonical Reads edges only for unique
compatible targets, aggregates deterministic exact resolved provenance, and
reports typed parser and resolver failures without placeholders. Production
emission remains limited to the parser's accepted forms. The confirmed multiline
decoder, private `Copied`/`CollapsedQuote`/`InsertedLf` mapping, raw negative
fixtures, typed unsupported-structure/virtual-table/temporary-table diagnostics,
semantic propagation, and full-builder all-or-nothing evidence are implemented.
Query identity, ownership, public API, accepted one-line behavior, resolver,
Writes, References, query-derived DependsOn, and Impact policy remain unchanged.

The separate registry-only `semantic_edge.reads` Coverage transition required
by ADR-0021 is complete. The next independent High task is Writes; this
investigation does not define or implement its semantics.

Readiness outcome: **Reads implementation and Coverage transition complete**.

## Rejected alternatives

1. Exact case-sensitive local-name lookup is rejected because ADR-0021 requires
   deterministic case normalization and it would make ordinary English and
   Russian case variants unresolved.
2. ASCII-only lowercase is rejected because it cannot normalize the accepted
   Russian identifier corpus.
3. Locale-sensitive casing is rejected because results would depend on runtime
   environment rather than graph input.
4. NFC, NFKC, or another Unicode normalization pass is rejected for the first
   slice because neither repository nor inspected platform evidence authorizes
   canonical or compatibility equivalence.
5. Full Unicode case folding or a new ICU dependency is rejected because it is
   not repository precedent, is not required for the accepted English/Russian
   slice, and has no proven 1C equivalence.
6. Treating every cross-kind normalized collision as ambiguous is rejected
   because the parsed namespace fixes the exact metadata kind and the existing
   graph resolver filters by exact kind before cardinality.
7. Extending `SemanticResolutionIndex` now is rejected because its public exact
   `EntityName` semantics are shared and source-independent.
8. Placing the resolver in `oneagent-bsl`, `oneagent-graph`, or
   `oneagent-analysis` is rejected because none owns both the EDT production
   phase and explicit workspace-scope evidence without weakening dependency
   boundaries.
9. Preserving indentation or the first `|` in decoded text is rejected because
   controlled platform probes show that neither contributes runtime characters.
10. Preserving physical source newline bytes unchanged is rejected because LF
    and CRLF source inputs both produce U+000A at every fragment boundary.

## Unknown and deferred behavior

The exact Unicode comparison algorithm used internally by the 1C platform for
query metadata identifiers remains **Unknown**. Therefore `str::to_lowercase()`
is an accepted deterministic OneAgent contract, not a claim of full platform
equivalence. Turkish/Azeri casing, Greek sigma equivalence, German sharp-s, and
other behavior outside the accepted English and Russian slice remain deferred;
the specified key still handles them deterministically and may conservatively
produce a missing or collision outcome.

Case-insensitive namespace variants, the Russian Information Register query
form, general expression grammar, multiline forms other than the tested pipe
style, comments, strings, and malformed-input recovery retain their existing
evidence status. JOIN, UNION, nesting, batches, temporary tables, and virtual
tables are recognized only for deterministic rejection; they are not accepted
grammar. The private decoded-to-BSL mapping remains an Accepted OneAgent model
rather than a claimed platform rule. ADR-0021 remains authoritative for graph
semantics. Production now
emits Reads only for the accepted fixture-backed set; `Writes`, `References`,
and query-derived `DependsOn` behavior is unchanged. `semantic_edge.reads` is
`Supported`; `semantic_edge.writes` remains `DeclaredOnly` as the only High EDT
gap. EDT Coverage is 1 High and 43 Medium gaps, while combined Coverage is
0 Critical, 1 High, and 44 Medium gaps.
