# Sprint 14 EDT/Designer XML conformance fixture

This fixture is the smallest tracked, non-empty paired slice accepted by
ADR-0036. It represents the same configuration, `DynamicSecurityOverridable`
Common Module, Common Module source, `FillSecurityCollection` Procedure, and
immediate ownership facts in EDT and hierarchical Designer XML layouts.

## Provenance and treatment

The EDT files originate from the registered ignored corpus
`/Users/maxim_tomshin/Development/1c-ssl-corpus/src/cf/OneAgent_EDTproject`.
The Designer files were produced from the registered ignored
`OneAgent_DesignerXML` corpus through a fresh temporary infobase with 1C:Enterprise
8.3.27.2214. Import attempt 1 of 2 failed because the source-preserving copy
excluded `Ext/ParentConfigurations/StandardSubsystemsLibrary.cf`. That exact
file was copied into the temporary source (SHA-256
`e443fbab718c7f50e55dbcd53b23699a274dd95f22dff7bda1ae8fefd1789402`), and
import attempt 2 of 2 succeeded without warnings. Designer then performed an
official hierarchical selective export of `Configuration`, `Catalog.Products`,
and `CommonModule.DynamicSecurityOverridable`, followed by an official
`-configDumpInfoOnly` export.

The EDT configuration and Common Module descriptors each apply one recorded,
parser-compatibility-only tag rewrite from the corpus' first Russian
`synonym/value` to the current EDT adapter's accepted `synonym/content`; text,
order, identity, and payload value are unchanged. All other retained files are
byte-for-byte copies, so their reduced hash is also their raw source/export
hash. The tracked reduction drops the
exported Product descriptor and Object/Manager modules, deferred root `Ext`
properties, and command modules because the executable paired first slice needs
only the shared configuration/Common Module facts. The full EDT Product
descriptor is also excluded because its member semantics are outside ADR-0036
and the current EDT adapter truthfully rejects a duplicate Attribute synonym.
`ConfigDumpInfo.xml` remains as the real producer marker; the test requests an
explicit partial build, so entries absent from this reduction create no
placeholder facts.

## Retained SHA-256 inventory

| Layout | Relative path | Raw and reduced SHA-256 |
|---|---|---|
| Designer | `ConfigDumpInfo.xml` | `36fe2f7314e08defd6c59688ed88a2133e97153841436d4ea7164cd7f54ce942` |
| Designer | `Configuration.xml` | `b7eed83a154d0f68c858f10d991ee985fb6d7df878f7abb328c1e441d57a2bdd` |
| Designer | `CommonModules/DynamicSecurityOverridable.xml` | `cafbab22d5a4494797aaf15b097d5118b22f60bf16e7017e147ce6048d482e3e` |
| Designer | `CommonModules/DynamicSecurityOverridable/Ext/Module.bsl` | `b798303db6df6427ac5e14abd616cf0838254e0262c22585b033950bb7642e48` |
| EDT | `.project` | `2e64e2459c496f324104296fa51ee53625a6271d6b96c1bcfd0814060438b435` |
| EDT | `src/Configuration/Configuration.mdo` | raw `017f5f4efeef37d63b72884d71a6770696763200c82eea3f2e0f38c634d3950c`; reduced `714994860f8f0ef62d63e846d07c1aa8bb9d558aa4f03a3dd6f3dd77fdc28507` |
| EDT | `src/CommonModules/DynamicSecurityOverridable/DynamicSecurityOverridable.mdo` | raw `e7d05be70e82e0fe421df2198b09556c4f729a35a6150d0f9d0f0471d4fcaa70`; reduced `5ee546609607467d0d3cf066e943b1b8c1c0bfb5885a9ed72d3e73284a0d0141` |
| EDT | `src/CommonModules/DynamicSecurityOverridable/Module.bsl` | `b56a39eedd53b8f621421e7e17dd59781ef3b6769e61f0e8b89c4192a7dac184` |

The canonical oracle excludes only adapter-specific source paths, producer
identifiers, XML vocabulary and serialization order, encoding/line endings,
and raw provenance. It compares stable UUID and owner-role identities, kinds,
exact names, accepted common payload, ownership, BSL declarations, explicit
partial terminal success, and public consumer/index results.
