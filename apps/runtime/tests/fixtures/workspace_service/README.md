# Runtime Workspace Service Fixture

This bounded tracked fixture is the public Runtime evidence root for ADR-0039.
Its `edt/` and `designer/` children are independently discoverable production
projects with different Configuration identities, so one Runtime build can
exercise both adapters without an artificial collision.

## Provenance and treatment

The EDT metadata sources and first `Posting` Procedure are exact copies of the
accepted `adapters/edt/tests/fixtures/writes_project/` reduction and retain its
detailed line-level provenance. The Object Module additionally contains one
generated `ReadMissingCatalog` Procedure over the accepted query syntax. Its
single missing Catalog reference deliberately produces one canonical terminal
request and recoverable diagnostic, so Runtime evidence proves preservation of
the request ledger as well as the legacy Writes observations. The `.project`
marker is derived from the tracked Sprint
14 conformance marker; only the Eclipse project name changes from `DNS_WE` to
`RuntimeWritesFixture`. The EDT Configuration remains generated fixture
scaffold with UUID `50000000-0000-0000-0000-000000000000` and name
`WritesFixture`.

The Designer root is the bounded `Complete` source set accepted by the adapter's
public complete-builder test. `ConfigDumpInfo.xml` and `Configuration.xml` are
source-derived reductions of the registered OneAgent Designer corpus described
in `docs/architecture/designer-xml-source-corpus.md`. The Common Module
descriptor is a field-preserving reduction of the tracked Sprint 14 descriptor;
the module is the same normalized source excerpt. The root retains configuration
UUID `408a41e7-907a-4fb3-8999-83d1e8b6e093`, exact name `DNSWorldEdition`,
Common Module UUID `dc24575c-a787-411d-93bd-494271291d73`, and exact accepted
procedure source. Omitted Designer metadata families are absent from the empty
bounded `ConfigVersions` inventory; Runtime still requests `Complete` and never
infers `Partial`.

All files are UTF-8 with LF endings. The local `.gitattributes` makes that
checkout property explicit. Negative public tests copy this root to a temporary
directory before applying one documented mutation; the tracked fixture is never
modified at test time.

## SHA-256 inventory

Hashes cover exact tracked bytes and are ordered by fixture-relative path.

| Relative path | SHA-256 |
| --- | --- |
| `.gitattributes` | `a79691a93b46e49ce460c26ef22afcc03d6eca1e63bf2edbc20e96159510f6c9` |
| `designer/CommonModules/DynamicSecurityOverridable.xml` | `2175b4fc7dbc7f7a4ff21a49beb49b4a25918b7950be012a0b1ad4dac02a8d6f` |
| `designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl` | `8c9d30d1227a42bd5e1f1aa4fec716bb065eb97ad2d0853c16513aee99f3dc92` |
| `designer/ConfigDumpInfo.xml` | `4df6aa78d875a562ca8749116ebc5ca9255db69a1d4bcecdf34c548d379600a0` |
| `designer/Configuration.xml` | `7e997994e30b731b2b22b735c26ca89289bb98a219340f196b954e5952a6b2de` |
| `edt/.project` | `dc579bbc2bd2c481b54d95799a4372db1923db1f9f41eb50f9b698c8f8a04916` |
| `edt/src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo` | `281845e55aca3a1b121c771b46c86ff96de497b325a47b5c329ae1c56d842e1c` |
| `edt/src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo` | `b6fefe78256e859a33c05be5b6d2cea61c8b5d00ce8275639591931542332cc9` |
| `edt/src/Configuration/Configuration.mdo` | `90096aa2eaf4514cd05c2fb3c1c711e7eee3a5aaaae2c7bfdb4cd2cdda99b9fb` |
| `edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl` | `2c264129f8fba3765652fe62868560b9555e1732e5124de5c5c34633a4deb1c7` |
| `edt/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo` | `54d82b4f8e678138c790b739dfd3c32c98b3417922fc77880df67f1beae6cc0c` |

## Accepted public observations

Sprint 41 safe-edit tests derive their temporary paired projects from the tracked
`adapters/designer-xml/tests/fixtures/sprint14_conformance/{edt,designer}` corpus
and this directory's complete Configuration/manifests. The second Common Module
copies the accepted `ExerciseSecurityCollection` declaration and changes its
existing call to a qualified call; its descriptor uses the next fixture UUID.
Query bodies come from the tracked EDT `reads_project` Object Module. Unsupported
and malformed variants derive from the tracked Query grammar/fixture, with BSL
multiline continuation preserved. The resolved variant copies the tracked
Products Catalog; an additional unchanged callable deliberately reuses the Query
binding. Unicode rename spellings, exact-size comment padding, and fault sentinels
are generated test data. Designer BOM/CRLF and EDT LF bytes are retained from the
paired corpus, independently of this directory's LF inventory above.

All copies, mutations, I/O fault artifacts, and temporary directories remain
under repository-local `local-artifacts/`. No private corpus or external project
is read. Each rejection checks actual production guards and original/applied
bytes; successful reversal checks exact byte restoration and immutable old Arcs.
Owner-local Analysis/adapter tests use canonical constructors and shared producer
encoders; they do not manufacture private Graph identities or parse ID strings.

- Production discovery finds exactly one EDT and one Designer XML root.
- Snapshot order follows canonical Configuration identity, not path order.
- EDT retains recoverable diagnostics, reference requests/statistics, and its
  graph report; Designer retains an independently validated graph with empty
  diagnostic and request evidence.
- Temporary mutations cover detector conflicts, duplicate Configuration
  identity, and fatal adapter input without introducing additional fixtures.
## Additional Task 5 boundary derivations

The operation admission corpus keeps the paired EDT descriptors and qualified
caller, renames the callable token to `A`, and repeats its direct call to produce
exactly 4096 or 4097 accepted occurrences. Short identifiers keep this boundary
test independent of large-document byte cost. The longer Unicode paired cases
also relocate the complete temporary Configuration beneath a custom child root.
Physical entry/raw/file/staging limits use temporary zero-filled files derived
from the same local fixture; none are tracked binary fixtures. Runtime tests
observe real I/O, builders, publication writers and policy gates at their owners.
