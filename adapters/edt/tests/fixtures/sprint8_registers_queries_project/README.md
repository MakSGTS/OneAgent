# Sprint 8 Register Query EDT Evidence Project

This reduced EDT project is the representative full-builder source for the
direct Accumulation and Accounting Register Query boundary accepted by
ADR-0030. Catalog and Information Register compatibility is reused from the
pre-existing `reads_project` fixture; those artifacts were not copied here.

The project preserves the confirmed real Common Module owners, callable names,
qualified persistent sources and aliases, Configuration declarations, target
metadata kinds, target names, and target UUIDs. It does not claim that the
reduced one-line Query programs are verbatim copies of the larger real Query
programs.

## Reduction contract

`Module.bsl` Procedure wrappers and Query construction statements are generated
scaffold. Each one-line Query text is `reduced-derived`: it keeps the real
qualified source and alias, while reducing projection and removing the real
`WHERE`, `ORDER BY`, parameter, and execution tails so the artifact stays
inside the already accepted complete one-source parser grammar.

Descriptor XML declarations recorded as `verbatim-fragment` preserve exact
individual source lines, but unrelated intervening descriptor content is
removed. XML declarations and closing tags are generated scaffold. The
Configuration UUID/name are fixture-only; its four selected child declarations
are verbatim individual source lines moved into the reduced descriptor.

Source ranges below are one-based and inclusive. Blob IDs are the current
`git hash-object` values of the complete real-source artifacts.

## Artifact manifest

| Fixture artifact and lines | Real source origin and blob | Treatment and preserved evidence | Expected production result |
|---|---|---|---|
| `src/AccountingRegisters/FinancialAccounting/FinancialAccounting.mdo:2-3` | `OneAgent_EDTproject/src/AccountingRegisters/FinancialAccounting/FinancialAccounting.mdo:2,12`; blob `e77313d4e396e23169560a8e320cc790b871bd46` | `verbatim-fragment`; exact `AccountingRegister` root, UUID `545e2fd3-9833-4a5f-a77c-122f76baa229`, and name `FinancialAccounting` | One exact top-level Accounting Register target for the QuerySource request. |
| `src/AccumulationRegisters/InventoryCost/InventoryCost.mdo:2-3` | `OneAgent_EDTproject/src/AccumulationRegisters/InventoryCost/InventoryCost.mdo:2,11`; blob `68177287166c2ae9e267d030737c6d4bb1b85f1d` | `verbatim-fragment`; exact `AccumulationRegister` root, UUID `3f1de785-2fe5-4a59-8998-b4f9b74f2c55`, and name `InventoryCost` | One exact top-level Accumulation Register target for the QuerySource request. |
| `src/CommonModules/Accounting/Accounting.mdo:2-4` | `OneAgent_EDTproject/src/CommonModules/Accounting/Accounting.mdo:2-3,8`; blob `89906c68e9daadd3bac62cc4b5a679c125e7f7c8` | `verbatim-fragment`; exact Common Module root/UUID, name, and server flag | Creates the real-format `Accounting` Common Module owner. |
| `src/CommonModules/Accounting/Module.bsl:1-4` | `OneAgent_EDTproject/src/CommonModules/Accounting/Module.bsl:936-970`, qualified source at line 961; blob `d2c4c6d1df21f7f06e10541472ac8d6e071bdfa9` | `reduced-derived`; preserves callable `InventoryCostBeforeWrite`, `AccumulationRegister.InventoryCost AS OldRecords`, and a compatible real projected field; wrapper/construction are generated | One stable Query owned by the Procedure, one resolved QuerySource request, one `Reads`, and one derived `DependsOn`. |
| `src/CommonModules/MonthEndTransactions/MonthEndTransactions.mdo:2-4` | `OneAgent_EDTproject/src/CommonModules/MonthEndTransactions/MonthEndTransactions.mdo:2-3,8`; blob `66ab7331c78a653b70e4b55dbf89675f58029011` | `verbatim-fragment`; exact Common Module root/UUID, name, and server flag | Creates the real-format `MonthEndTransactions` Common Module owner. |
| `src/CommonModules/MonthEndTransactions/Module.bsl:1-4` | `OneAgent_EDTproject/src/CommonModules/MonthEndTransactions/Module.bsl:1514-1541`, qualified source at line 1522; blob `da73ed9432b3160ab4f8de3e9b2d46aeb0e474a4` | `reduced-derived`; preserves callable `ARAPUpdateExecute`, `AccountingRegister.FinancialAccounting AS FinancialAccounting`, and the real projected field; wrapper/construction are generated | One stable Query owned by the Procedure, one resolved QuerySource request, one `Reads`, and one derived `DependsOn`. |
| `src/Configuration/Configuration.mdo:1-8` | Selected declarations from `OneAgent_EDTproject/src/Configuration/Configuration.mdo:1796,2115,3555,3583`; blob `df76375cd3e898a30988cd9dc344e01719e7a1cc` | `generated-scaffold` for root/UUID/name; four child declarations are `verbatim-fragment` | Makes the reduced project loadable and preserves the real Configuration membership evidence for both modules and targets. |

## Complete fixture integrity

| Artifact | Fixture SHA-256 |
|---|---|
| `src/AccountingRegisters/FinancialAccounting/FinancialAccounting.mdo` | `3bfe32f1adf1989eece135468bf4f963b758be8cbd96beafe75f6c00ac15d66a` |
| `src/AccumulationRegisters/InventoryCost/InventoryCost.mdo` | `64344f67b512b7e66b4b79959cba071b5010ac584f04fac0fcb3f83838676e6d` |
| `src/CommonModules/Accounting/Accounting.mdo` | `4527beebba5fec18e179cb008b6f8bce3b5db149c67af666bad4cea9c02b01a3` |
| `src/CommonModules/Accounting/Module.bsl` | `584a4b1f76e725dfafa3e5ef4c6b0a83f556779f078a972e652c1dbff0aa292d` |
| `src/CommonModules/MonthEndTransactions/MonthEndTransactions.mdo` | `b651fccb9b3167ec9fa0c61c0aea9e0fae98911b74387943f573ff2a99043992` |
| `src/CommonModules/MonthEndTransactions/Module.bsl` | `17ea5d1020161035e5660ab87c7afeb22e5e283f0e1e5f66484a5f7023872c2c` |
| `src/Configuration/Configuration.mdo` | `3dc8b83806cb592774a08fff6546bd358bd99d451e32aff95ac1df72a43633c7` |

## Deliberate omissions

Calculation Registers, register virtual tables, JOIN, UNION, nesting, batches,
temporary/external/parameter sources, broader expression grammar, new Query
declaration forms, register members/payload, Query mutation, write-derived
dependencies, and placeholder targets are absent. Missing, ambiguous,
incompatible, partial, duplicate, and parser-rejected outcomes remain covered
by focused generated tests because inventing real-source artifacts would add no
positive production provenance.
