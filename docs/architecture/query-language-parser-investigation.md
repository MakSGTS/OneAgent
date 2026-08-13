# Query-Language Parser Investigation

## Status and scope

This investigation defines the minimum evidence-backed 1C query-language
contract needed before implementing the first `Reads` slice accepted by
ADR-0021. It does not define a complete 1C query grammar and does not change BSL
extraction, graph construction, metadata resolution, or edge emission.

Evidence labels used below are:

- **Confirmed by real repository source**: visible in a verified range under
  `OneAgent_EDTproject/src`;
- **Confirmed by an existing repository test**: asserted by committed Rust test
  code;
- **Accepted by ADR-0021 but not yet represented by source evidence**: required
  policy without a repository-owned raw syntax example;
- **Unknown**: repository evidence is insufficient to choose a contract.

## Current `BslQuery` boundary

`crates/bsl/src/queries.rs:7-16` defines `BslQuery` as an existing Query identity,
owner, local binding, static text, and one-based BSL declaration line.
`BslQuery::text()` returns the stored text (`crates/bsl/src/queries.rs:63-67`).
The current line-oriented extractor accepts a static string in a supported
constructor or `.Text` assignment (`crates/bsl/src/queries.rs:317-360`) and its
literal decoder only handles a quoted value available on one BSL source line
(`crates/bsl/src/queries.rs:375-397`).

This is BSL Query extraction, not 1C query-language parsing. The extractor does
not tokenize `SELECT`, classify statements, discover sources, build an AST, or
validate query-language syntax. Dynamic or ambiguous BSL assignments are omitted
before a Query node exists, as shown by `crates/bsl/src/queries.rs:518-552` and
the production-path test at `adapters/edt/src/lib.rs:3561-3584`.

The production graph path reads modules and invokes `LineBslQueryExtractor`
(`adapters/edt/src/bsl_graph.rs:140-167`), then emits each accepted Query node and
its `Contains` edge (`adapters/edt/src/bsl_graph.rs:306-329`). The existing
production test proves stable Query identity and ownership and explicitly proves
that no `Reads`, `Writes`, or query-derived `DependsOn` edges are emitted
(`adapters/edt/src/lib.rs:3461-3558`). Query-language parsing must attach analysis
to this identity; it must not replace or split the Query node.

### Multiline BSL string limitation

Real query programs commonly use multiline BSL strings whose source lines begin
with `|`. The current extractor has no multiline decoding contract. Repository
evidence inspected in this investigation does not establish whether and how
continuation markers, indentation, doubled quotes, or source newlines map to the
raw query text that a future parser should receive. Consequently, this document
cites those BSL ranges as syntax evidence but does not claim that removing the
wrapper syntax produces `BslQuery::text()`. No raw fixture was manufactured from
such a range.

## Evidence inventory

| Finding | Evidence | Status | First-slice consequence |
|---|---|---|---|
| Direct English Catalog source | `OneAgent_EDTproject/src/Catalogs/AdditionalReportsAndDataProcessors/Forms/QuickAccessToAdditionalReportsAndDataProcessors/Module.bsl:140-144` contains one static line with `FROM Catalog.Users`; `crates/bsl/src/queries.rs:455-474` and `adapters/edt/src/lib.rs:3461-3477` use `Catalog.Products` | Confirmed by real repository source and an existing repository test | `Catalog.<Name>` is repository-backed |
| Direct English Information Register source and alias | `OneAgent_EDTproject/src/CommonModules/MarkedObjectsDeletionInternal/Module.bsl:764-771` contains `FROM InformationRegister.ObjectsToDelete AS Tab` | Confirmed by real repository source | `InformationRegister.<Name>` and an optional `AS` alias are repository-backed |
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

## Identifier and case evidence

Evidence establishes dotted qualified names and aliases with the exact spellings
shown in the inventory. It does not establish quoted identifiers, non-ASCII
metadata identifiers in real query source, or identifier character rules.

The BSL extractor lowercases BSL constructor/property keywords internally, but
that is not evidence for query-language case semantics. The inspected
query-language examples use uppercase keywords and consistently cased namespace
and metadata names. ADR-0021 requires deterministic case normalization, but the
repository does not prove whether query keywords, namespaces, or identifiers are
case-insensitive or which Unicode mapping applies. Case normalization therefore
remains **Unknown** and must be decided and fixture-backed before metadata
resolution is implemented. Raw spelling must be retained regardless of that
decision.

## Source-location alternatives

| Alternative | Repository evidence | Decision |
|---|---|---|
| One-based line and column in raw query text | Existing BSL models expose a one-based BSL declaration line, but no query-text column or multiline mapping | Insufficient |
| Unicode scalar offsets | Rust code uses character iteration while decoding one-line BSL literals, but exposes no offset contract | Insufficient |
| UTF-8 byte offsets | Rust `str` slicing uses UTF-8 byte indices internally, but no public source-position model relies on them | Insufficient |

No source-location contract can be recommended safely now. The parser task is
blocked on choosing a raw-query coordinate system, including whether ranges are
half-open, whether line/column values are zero- or one-based, how CRLF is
normalized, and how a raw-query range maps back to a BSL literal. The choice must
be tested with ASCII and Russian fixtures before typed diagnostics or provenance
depend on it. The existing one-based `BslQuery::line()` may identify the wrapper
declaration, but it cannot substitute for a query-text location.

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
| Metadata resolution | Missing metadata target | An allowlisted parsed source has no compatible graph target |
| Metadata resolution | Ambiguous metadata target | More than one compatible target remains, with deterministically ordered candidates |
| Metadata resolution | Incompatible metadata target kind | A name resolves only at a disallowed kind |

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

No fixture is described as a minimized or verbatim decoding of multiline BSL
text. Missing raw fixtures for scalar parameters, comments, keyword-like string
literals, delimiters/batches, joins, unions, nested queries, temporary tables,
virtual tables, and malformed syntax are explicit evidence gaps.

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
| Statement delimiter and batch detection | Confirmed only in multiline BSL source | None | Multiple statements | Unsupported query structure | Raw fixture; single trailing-delimiter policy |
| `JOIN` detection | Confirmed only in multiline BSL source | None | Multiple top-level sources | Unsupported query structure | Raw fixture and join grammar boundary |
| `UNION` detection | Confirmed only in multiline BSL source | None | Multiple branches | Unsupported query structure | Raw fixture |
| Nested-query detection | Confirmed only in multiline BSL source | None | Nested source scope | Unsupported query structure | Raw fixture and balanced-delimiter grammar |
| Temporary-table detection | Confirmed only in multiline BSL source | None | Temporary declaration/source | Temporary table | Raw fixture |
| Virtual-table detection | Confirmed only in multiline BSL source | None | Virtual table source | Virtual table source | Raw fixture and invocation grammar |
| Malformed static input | Accepted by ADR-0021, not source-evidenced | None | No complete parsed program | Malformed query syntax | Evidence-backed malformed corpus and recovery policy |
| Deterministic query-text location | Unknown | All future diagnostic fixtures | Location attached to token/source/diagnostic | Applicable typed diagnostic | Coordinate and BSL mapping contract |
| Case normalization | Unknown | None | Preserved raw spelling plus normalized lookup key | Resolver diagnostic only after parsing | Case and Unicode normalization policy |

## Readiness conclusion and next task boundary

The repository contains enough evidence to begin a deliberately narrow parser
for the two ADR-0021 target categories in English: Catalog and Information
Register. It also contains test-backed Russian `ВЫБРАТЬ`, `ИЗ`, and `Справочник`
forms and a non-query metadata-type mapping for `РегистрСведений`. It does not
contain sufficient evidence to claim a complete bilingual
namespace contract, general expression grammar, case policy, multiline BSL
decoding, or deterministic query-text locations.

The next parser implementation task may introduce a private typed lexer/parser
and fixture-driven tests for the four raw fixtures in this corpus, provided it:

- consumes the complete input;
- keeps the source-location decision explicit and resolves it before asserting
  typed diagnostic locations;
- returns a complete-source proof only for the exact accepted fixture shapes;
- returns the typed parameter-source diagnostic for the unsupported fixture;
- rejects all unimplemented tails and structures instead of extracting a partial
  source;
- does not add metadata resolution, graph validation, or `Reads` emission.

Before expanding that slice, add repository-backed raw fixtures for comments,
strings, scalar parameters, statement boundaries, joins, unions, nesting,
temporary tables, virtual tables, malformed input, case variants, and the Russian
Information Register form. ADR-0021 remains the authority for graph semantics;
this investigation does not infer `Reads`, `Writes`, `References`, or
`DependsOn` behavior beyond it.
