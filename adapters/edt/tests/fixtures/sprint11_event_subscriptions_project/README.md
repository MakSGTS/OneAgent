# Sprint 11 Event Subscription EDT Evidence Project

This tracked reduced EDT project is the representative production-builder
source for the Event Subscription boundary accepted by ADR-0033. It preserves
one exact supported selector, two equivalent bare manager/object selectors,
one unsupported selector, exported and non-exported Procedure targets, and
live event and synonym values. The fixture is self-contained and does not
depend on the root-ignored `OneAgent_EDTproject/` tree at test time.

## Reduction contract

`verbatim-fragment` means the selected XML lines are copied unchanged.
`reduced-derived` means the root kind, UUID, name, selected fields, or BSL
declaration are copied while unrelated content and procedure bodies are
removed. `recomposed-evidence` means individually live-proven fragments are
combined to exercise one accepted semantic rule that no single live descriptor
contains. XML declarations and closing tags are generated scaffold unless
listed as selected evidence. Source ranges are one-based and inclusive.

The live corpus has 99 Event Subscription descriptors and 93 unique handler
paths. A complete multiline declaration audit shows that all 93 referenced
handlers are exported. The earlier line-oriented audit misclassified four
unique multiline declarations as non-exported because `Export` occurs on the
final declaration line. The non-exported positive case below therefore
recomposes the real `DataHistoryManagement.DeleteOldDataHistoryVersions`
Procedure as a handler target. This preserves the accepted export-agnostic
ownership rule without falsely claiming that the recomposed handler path is a
verbatim live Event Subscription field.

Source blob IDs are read-only `git hash-object` values. Source SHA-256 values
cover the complete ignored artifacts; fixture SHA-256 values cover the exact
tracked reductions.

## Live-source artifact manifest

| Fixture artifact | Live source origin and selected lines | Source hashes | Treatment and preserved evidence |
|---|---|---|---|
| `src/Configuration/Configuration.mdo` | `OneAgent_EDTproject/src/Configuration/Configuration.mdo:2-3` | blob `df76375cd3e898a30988cd9dc344e01719e7a1cc`; SHA-256 `017f5f4efeef37d63b72884d71a6770696763200c82eea3f2e0f38c634d3950c` | `reduced-derived`; exact root kind, UUID, and name provide the production owner. |
| `src/Catalogs/Products/Products.mdo` | `OneAgent_EDTproject/src/Catalogs/Products/Products.mdo:2,10` | blob `0c20e9811251df36fafb07fe1316c5c7f1d86f79`; SHA-256 `917d9aeb244e71660849cb83bb4b27c1934d1d642561ce01ef8cc245c3742228` | `reduced-derived`; exact Catalog UUID and name provide the shared family target. |
| `src/BusinessProcesses/Job/Job.mdo` | `OneAgent_EDTproject/src/BusinessProcesses/Job/Job.mdo:2,11` | blob `894fe026d13b2dcd9d78674cccdddf63523dd055`; SHA-256 `277bccaaeb3efe22f65ca9f6a7ab5f856f47c47dad14e4270c189ee9afcc72e0` | `reduced-derived`; exact Business Process UUID and name provide the qualified target. |
| `src/CommonModules/DataHistoryManagement/DataHistoryManagement.mdo` | `OneAgent_EDTproject/src/CommonModules/DataHistoryManagement/DataHistoryManagement.mdo:2-3,8` | blob `a86b2b8459f40890615d38ccad20d8346b381b4d`; SHA-256 `e610d331535fe9be2f14c4026e727bf399a10c40b1a3d886e2acbd7bb88e318b` | `verbatim-fragment`; exact Common Module UUID, name, and server flag. |
| `src/CommonModules/DataHistoryManagement/Module.bsl` | `OneAgent_EDTproject/src/CommonModules/DataHistoryManagement/Module.bsl:195,200,208,222` | blob `2aa7e4dc5afa950e417e6ae286d7ff0830b37239`; SHA-256 `0287e9bfb671d681dd0dd93c533f5ad1fe4b31afbd1f79f315778bca2cc5ea32` | `reduced-derived`; exact exported and non-exported Procedure declarations with empty generated bodies. |
| `src/CommonModules/BusinessProcessesAndTasksClientServer/BusinessProcessesAndTasksClientServer.mdo` | `OneAgent_EDTproject/src/CommonModules/BusinessProcessesAndTasksClientServer/BusinessProcessesAndTasksClientServer.mdo:2-3,13` | blob `6a574d708fb6653bd4604fa7b65bca479b5b1a9f`; SHA-256 `b5cd29cc7fa303ca16c1c8d4ba73d9308e8f6997caea08fb357adfdd03bbdef6` | `verbatim-fragment`; exact Common Module UUID, name, and server flag. |
| `src/CommonModules/BusinessProcessesAndTasksClientServer/Module.bsl` | `OneAgent_EDTproject/src/CommonModules/BusinessProcessesAndTasksClientServer/Module.bsl:24,30` | blob `d880fb5f3489672992d4a9db61b1c597a4e7fa34`; SHA-256 `cdb35e289e8d0895c3686796cca7e133797eb6a49da4f834d775cbfcca36f235` | `reduced-derived`; exact exported Procedure declaration with an empty generated body. |
| `src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo` | `OneAgent_EDTproject/src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo:2-8,10,12-13`; plus `OneAgent_EDTproject/src/EventSubscriptions/Catalogs_BeforeWrite/Catalogs_BeforeWrite.mdo:9` | blobs `f6b4b8febb4247c105a992ac635749a1f75d539d`, `ff5468dcd07ed72dc546c94809e443156984a236`; SHA-256 `4d1799e5a98a635949fac0b19cd2237f17859b0ce7a910afd52ae65bf91b0fa4`, `bb0203c1c992d891ba42d0830298da126456952e1114c553b2facaa16833c137` | `recomposed-evidence`; exact identity, synonym, `CatalogManager`, event, and exported handler plus the exact `CatalogObject` observation. Both selectors resolve `Products` and aggregate into one References edge with two provenance records. |
| `src/EventSubscriptions/GetBusinessProcessPresentationFields/GetBusinessProcessPresentationFields.mdo` | `OneAgent_EDTproject/src/EventSubscriptions/GetBusinessProcessPresentationFields/GetBusinessProcessPresentationFields.mdo:1-17` | blob `36342face750a3645c4a098d07e918a72878fe43`; SHA-256 `81ea52e17baba044b68d2c85860701eac20c8c217597ad38a83eec64f6d52c99` | `verbatim-fragment`; exact qualified manager selector, multilingual synonyms, event, and exported handler. |
| `src/EventSubscriptions/CheckAccessBeforeWriteRecordsSet/CheckAccessBeforeWriteRecordsSet.mdo` | `OneAgent_EDTproject/src/EventSubscriptions/CheckAccessBeforeWriteRecordsSet/CheckAccessBeforeWriteRecordsSet.mdo:2-15`; handler components from `OneAgent_EDTproject/src/CommonModules/DataHistoryManagement/DataHistoryManagement.mdo:3` and `Module.bsl:208` | blobs `01f9813a54ba6b6d925e428411fb88eccea4b30f`, `a86b2b8459f40890615d38ccad20d8346b381b4d`, `2aa7e4dc5afa950e417e6ae286d7ff0830b37239`; SHA-256 `100a1f1c98bda23d6f6f41d8e0b501eff164d7bdde2b4781787335dd7b2163f9`, `e610d331535fe9be2f14c4026e727bf399a10c40b1a3d886e2acbd7bb88e318b`, `0287e9bfb671d681dd0dd93c533f5ad1fe4b31afbd1f79f315778bca2cc5ea32` | `recomposed-evidence`; exact subscription identity, synonyms, unsupported selector, and event retargeted to an exact live non-exported owned Procedure. |

## Complete fixture integrity

| Artifact | Fixture SHA-256 |
|---|---|
| `src/BusinessProcesses/Job/Job.mdo` | `eb9ee98694d2ec96739f9459f822608c26a73fce3bd19b4d6e0a5282e151f756` |
| `src/Catalogs/Products/Products.mdo` | `43e49c565336404ea6d9bdb142ba8d9d2fc0918f24f39c51405ef9b9be86d644` |
| `src/CommonModules/BusinessProcessesAndTasksClientServer/BusinessProcessesAndTasksClientServer.mdo` | `ffa83339b1161d300e77da31fec6037139eb9551046f51581b191cead5df1bca` |
| `src/CommonModules/BusinessProcessesAndTasksClientServer/Module.bsl` | `a83c0704df9464b6126b64567837a06fac536e47a16864affef1ab28a41e2ccf` |
| `src/CommonModules/DataHistoryManagement/DataHistoryManagement.mdo` | `7b7a16fc2a859eb955ea3bd63850165bc9ada76bd8a4c388adf8a778d3db02b2` |
| `src/CommonModules/DataHistoryManagement/Module.bsl` | `23cf23538d0b4be79058e8353e7fc8d78895e6bdc169e3adc0b46532f5573c05` |
| `src/Configuration/Configuration.mdo` | `aa1258a328ce01b4c149ad2e2d52eb7243da125b15599dfdec4048dc4003977d` |
| `src/EventSubscriptions/AfterWriteDataHistoryVersionsProcessing/AfterWriteDataHistoryVersionsProcessing.mdo` | `24dfe7c88255e487f1fe49a7148fb43b941ef94ed84f453639570229d44cb136` |
| `src/EventSubscriptions/CheckAccessBeforeWriteRecordsSet/CheckAccessBeforeWriteRecordsSet.mdo` | `8b1dbf262e9b827128a230a55b255de1abd6a282e8a8a981de1ddd68b497987d` |
| `src/EventSubscriptions/GetBusinessProcessPresentationFields/GetBusinessProcessPresentationFields.mdo` | `81ea52e17baba044b68d2c85860701eac20c8c217597ad38a83eec64f6d52c99` |

## Deliberate omissions

The reduction omits unrelated metadata fields, procedure bodies, additional
Catalog and Business Process instances, unsupported metadata entities, and all
runtime dispatch behavior. Missing, ambiguous, incompatible, malformed,
Function-handler, source/handler retarget, and add/remove transition matrices
remain generated evidence. Multi-target selectors remain outside the public
ADR-0024 request ledger, and Triggers remains outside dependency and Impact
propagation.
