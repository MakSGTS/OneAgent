# Sprint 12 Report Data Composition EDT Evidence Project

This tracked reduced EDT project is the representative production-builder
source for the Report Data Composition boundary accepted by ADR-0034. It is
self-contained and has no test-time dependency on the root-ignored
`OneAgent_EDTproject/` tree.

## Reduction contract

`reduced-derived` preserves the selected live identity and semantic values
while removing unrelated metadata and DCS settings. XML declarations, closing
tags, indentation, and the minimum Configuration shell are generated scaffold.
`verbatim-content` preserves the decoded source text while normalizing XML
indentation. Source ranges are one-based and inclusive. Source blob IDs are
read-only `git hash-object` values; source SHA-256 values cover complete ignored
artifacts, and fixture SHA-256 values cover exact tracked reductions.

The fixture keeps Query, Object, Union, valid empty main, valid non-main,
nested-data-set deferred, and field-folder deferred source shapes. It does not
run DCS Query text through the BSL Query parser and intentionally emits no
QuerySource request, `Reads`, `DependsOn`, or `References` fact.

## Live-source artifact manifest

| Fixture artifact | Exact live source and selected lines | Source hashes | Treatment and preserved evidence |
|---|---|---|---|
| `src/Configuration/Configuration.mdo` | `OneAgent_EDTproject/src/Configuration/Configuration.mdo:2-3` | blob `df76375cd3e898a30988cd9dc344e01719e7a1cc`; SHA-256 `017f5f4efeef37d63b72884d71a6770696763200c82eea3f2e0f38c634d3950c` | `reduced-derived`; exact Configuration UUID and name. |
| `src/Reports/AccessGroupsMembers/AccessGroupsMembers.mdo` | `OneAgent_EDTproject/src/Reports/AccessGroupsMembers/AccessGroupsMembers.mdo:2,7,18,22-23,32` | blob `feead60bbc008a02683741fda7ef0a4203cb833f`; SHA-256 `62c54bba852c42a98516fe47df3de4ba6bdaf6e4b7cde461a9aa0d3d9644afa3` | `reduced-derived`; exact Report/schema UUIDs, names, type, and main selection. |
| `src/Reports/AccessGroupsMembers/Templates/Template/Template.dcs` | `OneAgent_EDTproject/src/Reports/AccessGroupsMembers/Templates/Template/Template.dcs:2-12,73-95` | blob `2db062fd2d25d14cc268b64928607fd1b5024494`; SHA-256 `78a0c1e4536a71e0fdcd875c2dc65e671f0f1db096dab7f8847dca5b444af834` | `verbatim-content`; exact root/source, Query data-set name, first field/path, and complete decoded Query text with normalized indentation. |
| `src/Reports/VolumeIntegrityCheck/VolumeIntegrityCheck.mdo` | `OneAgent_EDTproject/src/Reports/VolumeIntegrityCheck/VolumeIntegrityCheck.mdo:2,7,17,19-20,29` | blob `7438d04194d9bbbd16adb8bbbd6282be0e2b0e8f`; SHA-256 `475ee0e3b42db5049aeadc58094c4db2f02ca9a64da0399ee89e30d8ccfbbf07` | `reduced-derived`; exact Report/schema identity and main selection. |
| `src/Reports/VolumeIntegrityCheck/Templates/MainDataCompositionSchema/Template.dcs` | `OneAgent_EDTproject/src/Reports/VolumeIntegrityCheck/Templates/MainDataCompositionSchema/Template.dcs:2-11,149-151` | blob `83bafc3f6e7a2f81cfbde5563b54b4566a6803f8`; SHA-256 `f3a168f831bba28fe1a5d0ebe06ee3054581beeb31564f080629af561e0e4173` | `reduced-derived`; exact Object data-set name, local source, first field/path, and object name. |
| `src/Reports/AccountCardFinancialAccounting/AccountCardFinancialAccounting.mdo` | `OneAgent_EDTproject/src/Reports/AccountCardFinancialAccounting/AccountCardFinancialAccounting.mdo:2,7,19,27-28,49` | blob `b9cced358e90d0787a9edac1b67121df9094bdf1`; SHA-256 `58da2f21cc9f8d6d4b24e2d6fb0f6622b9a1626a3f8c960e7a95d93a26630bcd` | `reduced-derived`; exact Report/schema identity and main selection. |
| `src/Reports/AccountCardFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `OneAgent_EDTproject/src/Reports/AccountCardFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs:2-11` | blob `051149db58619cb186c31e0402826fe54973f985`; SHA-256 `0283c13aac800107813da1a266faec774c2cdd3bc2c13c6bac5f0d0658be6354` | `reduced-derived`; exact direct Union name and first named field/path. |
| `src/Reports/ControlOfProductsAccounting/ControlOfProductsAccounting.mdo` | `OneAgent_EDTproject/src/Reports/ControlOfProductsAccounting/ControlOfProductsAccounting.mdo:2,7,18,21-22,27` | blob `97f5cf9238e8d4f0205bd8bf77529da65cf28e1c`; SHA-256 `c0bbc8f9a5bfae23f7e2cbbd29ba2df89ab52ba6591a93ec5445b999f97934cf` | `reduced-derived`; exact Report/schema identity and main selection. |
| `src/Reports/ControlOfProductsAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `OneAgent_EDTproject/src/Reports/ControlOfProductsAccounting/Templates/MainDataCompositionSchema/Template.dcs:2-11,156-157,473,481-488,613-614` | blob `b9416ada79bea360d08d3a1e130c5c1c8d85aab9`; SHA-256 `d40f78baf578e7cc41039a2365af493a97575d2f1d99ae499e496c8f48bdbd19` | `reduced-derived`; exact direct Query identity/field plus one live nested-schema duplicate-name Query shape; opaque Query bodies are reduced to their exact non-empty `SELECT ALLOWED` prefix. |
| `src/Reports/UniversalReport/UniversalReport.mdo` | `OneAgent_EDTproject/src/Reports/UniversalReport/UniversalReport.mdo:2,7,18,22-23,32` | blob `c8352649118b27f63449034369306d0a21a9c471`; SHA-256 `af48b2f357e91c05dd100acf598023135f9b35a134664bbcd43c6fa47d9c8ef9` | `reduced-derived`; exact Report/schema identity and main selection. |
| `src/Reports/UniversalReport/Templates/MainDataCompositionSchema/Template.dcs` | `OneAgent_EDTproject/src/Reports/UniversalReport/Templates/MainDataCompositionSchema/Template.dcs:2` | blob `c19556a1a84b6145bcdd82158a3ce5b868f6eca2`; SHA-256 `f290dde5b2c100e7be3750ae92410a60324c9b26f857a0998d88636803de3c94` | `reduced-derived`; exact valid DCS root with all unrelated parameter content removed, proving an empty accepted entity slice. |
| `src/Reports/FinancialReport/FinancialReport.mdo` | `OneAgent_EDTproject/src/Reports/FinancialReport/FinancialReport.mdo:2,7,299-300,321` | blob `ad25011574b32a789bd1ca579e8403b1fe2e409b`; SHA-256 `fb3acb427e48017030d65157ec14be452198067182c94f033708b8131647d459` | `reduced-derived`; exact Report/schema identity with no main selection. |
| `src/Reports/FinancialReport/Templates/DerivedItemOperands/Template.dcs` | `OneAgent_EDTproject/src/Reports/FinancialReport/Templates/DerivedItemOperands/Template.dcs:2-10,394-416` | blob `4f8afe90090c59a689b3903347b367e52c1828ee`; SHA-256 `9360b3811ed2ebbc40e5b0c4c81ebb1bec8d581b696afb32623e5369e4a05ad6` | `verbatim-content`; exact Query data-set identity, first field/path, source, and complete decoded Query text with normalized indentation. |
| `src/Reports/AccountingReportFinancialAccounting/AccountingReportFinancialAccounting.mdo` | `OneAgent_EDTproject/src/Reports/AccountingReportFinancialAccounting/AccountingReportFinancialAccounting.mdo:2,7,31,39-40,61` | blob `ad1f02f2f0470e437e4d9d37f6d9da3c6f4551e7`; SHA-256 `0b78ae984a73759c451de5b5bcf81fffce53d762f76d44b7ff3c0c2384dfa8b7` | `reduced-derived`; exact Report/schema identity and main selection. |
| `src/Reports/AccountingReportFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `OneAgent_EDTproject/src/Reports/AccountingReportFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs:2-11,790-793` | blob `30d490d96b486835124998f645e320a2070acb1a`; SHA-256 `b191933ae7b4f8437075e64331889c62e18d781724b1dcac816c48d8e41602fa` | `reduced-derived`; exact direct Union/field identity plus one exact field-folder type and path. |

## Complete fixture integrity

| Artifact | Fixture SHA-256 |
|---|---|
| `src/Configuration/Configuration.mdo` | `aa1258a328ce01b4c149ad2e2d52eb7243da125b15599dfdec4048dc4003977d` |
| `src/Reports/AccessGroupsMembers/AccessGroupsMembers.mdo` | `7e84f9a473134e8ce696b4c817a4b5aaecf04da3b54bba159ab08d6c4504520f` |
| `src/Reports/AccessGroupsMembers/Templates/Template/Template.dcs` | `ea4d44e5b132bbcb9faac66526d3cd77d023ce256d8b3ab2c224b86403844251` |
| `src/Reports/AccountCardFinancialAccounting/AccountCardFinancialAccounting.mdo` | `067d5f0d81228c525a8efa66e9a9eebd60781fb1f5c5e69cf0f46c1700544912` |
| `src/Reports/AccountCardFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `3a8b6efe35b6a24653addf3392650c0b74491e920260ad15c18fc33d0b804dd7` |
| `src/Reports/AccountingReportFinancialAccounting/AccountingReportFinancialAccounting.mdo` | `be172267bd7bab8f47f5e7cdf81e007b95fdaf33c589f5707c1c6a3043947369` |
| `src/Reports/AccountingReportFinancialAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `90d17e951069c55214f02aa6370edd3dcb7534a4be92b99f4a18c916f5a47286` |
| `src/Reports/ControlOfProductsAccounting/ControlOfProductsAccounting.mdo` | `d642fec5bf95edf88b2ba982a00b89da0ec409ddb4249bdd523fa263d92f1858` |
| `src/Reports/ControlOfProductsAccounting/Templates/MainDataCompositionSchema/Template.dcs` | `8b3724e495b69f802beb69177bfd9397a86813f671949e8a64e92af3c49b9ced` |
| `src/Reports/FinancialReport/FinancialReport.mdo` | `961cb4994df1eccc8b4179dfda07372b5c78fba7fdffbec89b886b168dc5cb75` |
| `src/Reports/FinancialReport/Templates/DerivedItemOperands/Template.dcs` | `e3ef4c57b7e45a315cdc85a84c0389c37c79226aab5bcc2e786dfaa0fd19dccb` |
| `src/Reports/UniversalReport/Templates/MainDataCompositionSchema/Template.dcs` | `61a70b237f2795e260a88c049308b35fe97bd563b899e533702eacb0ec7c5aa4` |
| `src/Reports/UniversalReport/UniversalReport.mdo` | `0e53c0bf4328cf2cbe04964d415b3cee3fb024b345187778c09bae7c113fae6f` |
| `src/Reports/VolumeIntegrityCheck/Templates/MainDataCompositionSchema/Template.dcs` | `c36d5f43d012ce5f178c6b5a00a0c74e247f9020e39742fd1d61082ee6e88201` |
| `src/Reports/VolumeIntegrityCheck/VolumeIntegrityCheck.mdo` | `2a5b03640c3e7cea4f0fd927a8a6968ae8567644cb192efc7623143d711f102e` |

## Deliberate omissions

The reduction omits unrelated Report members/modules/forms/commands, DCS
settings and presentation content, extra direct fields, extra nested schemas,
five of six live field folders, and seven of eight live nested data sets.
Generated tests retain fatal-source, add/remove/modify, ownership, and reordered
transition coverage. Nested entities, field folders, DCS Query parsing, virtual
tables, batches, temporary tables, lineage, runtime composition, non-Report
schemas, placeholders, and partial source relations remain deferred.
