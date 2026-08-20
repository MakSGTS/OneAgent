# Sprint 10 Subsystem Hierarchy EDT Evidence Project

This tracked reduced EDT project is the representative full-builder source for
the nested Subsystem hierarchy and composition boundary accepted by ADR-0032.
It preserves a repository-observed five-level chain, two `Bank` local names
under different parents, shared direct content, nested direct content, and one
observed deferred Subsystem self-content token.

The live origin is the root-ignored `OneAgent_EDTproject/` tree. The fixture is
self-contained and remains auditable when that tree is absent or changes.

## Reduction contract

Every Subsystem root kind, UUID, direct name, selected `content`, selected
`subsystems`, and `parentSubsystem` field is derived from the exact live source
line recorded below. Unselected siblings, content, presentation fields, and
other descriptor data are removed. XML declarations, closing tags, and the
Configuration descriptor are generated scaffold. Namespace declarations that
are unused by the selected fragment are removed. The source files for
`InterfaceFinancialAccounting` use CRLF; selected fields are newline-normalized
to LF in this fixture.

`verbatim-fragment` means the complete selected line is preserved after the
document-level newline normalization. `reduced-derived` means the semantic XML
root kind/UUID/name is preserved while unused namespace declarations or
intervening source content are removed. Source ranges are one-based and
inclusive. Blob IDs are `git hash-object` values of the complete live artifacts;
source SHA-256 values make the ignored input independently auditable.

## Live-source artifact manifest

| Fixture artifact | Live source origin and selected lines | Source hashes | Treatment and preserved evidence |
|---|---|---|---|
| `src/Subsystems/DNSCore/DNSCore.mdo` | `OneAgent_EDTproject/src/Subsystems/DNSCore/DNSCore.mdo:2-3,17,27` | blob `76d5b4a3285892a188012a89cb4f0cef30a27fa5`; SHA-256 `a559109e6c9099fdcb60a67a86fb871e3a17ea9efa679b0a96688628cdd794a2` | `verbatim-fragment`; exact root/UUID/name, shared `CommonModule.AddressableStorage` content, and direct `Common` child. |
| `src/Subsystems/DNSCore/Subsystems/Common/Common.mdo` | `OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Common.mdo:2-3,144,205,220` | blob `2f36391bacd940dc2ac8865ebac810a06dd1dff1`; SHA-256 `44547cf0a60924660f806e12c0e0adcee42c01af95e5fb995b5a26b922935fbe` | `verbatim-fragment`; depth 2, the same direct member, direct child, and complete qualified parent. |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/FinancialAccounting.mdo` | `OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/FinancialAccounting.mdo:2-3,147-148` | blob `e161548cca4bd3d2ba250e4e19e616f3a6a15a86`; SHA-256 `9944ce1f0859aa37dd1a1ab08dbc7e7b962230793058ef4912163af8b0400745` | `verbatim-fragment`; depth 3 and direct `Treasury` relation. |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Treasury.mdo` | `OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Treasury.mdo:2-3,24,26` | blob `d671ef6c7c27cd7f3b3b4bd27d1a20b21fed35ae`; SHA-256 `908a67eb283733b7bb4a2e2dc690764374e11f2cd73ffdee44292b5f62eb2afd` | `verbatim-fragment`; depth 4 and direct `Bank` relation. |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Subsystems/Bank/Bank.mdo` | `OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Subsystems/Bank/Bank.mdo:2-3,16,61` | blob `4404d07a3e9722f1dc688fdea585d2b0cba01d39`; SHA-256 `8ff7df9101236468edc8dcc8d42be759172d1d468f2a4857d727c0e7f66deddd` | `verbatim-fragment`; depth 5, nested `Document.BankAccountIssue` content, and complete qualified parent. |
| `src/Subsystems/InterfaceFinancialAccounting/InterfaceFinancialAccounting.mdo` | `OneAgent_EDTproject/src/Subsystems/InterfaceFinancialAccounting/InterfaceFinancialAccounting.mdo:2-3,34` | blob `5a1afa50445b1d2cf00868be8fcc2ae84e75b37f`; SHA-256 `556b7089e1037546080887c64910ab6bc11ba1cad1f318cdc2fed1106db61b42` | `reduced-derived`; exact root kind/UUID/name and direct `Bank` child, with unused namespaces removed. |
| `src/Subsystems/InterfaceFinancialAccounting/Subsystems/Bank/Bank.mdo` | `OneAgent_EDTproject/src/Subsystems/InterfaceFinancialAccounting/Subsystems/Bank/Bank.mdo:2-3,14,17` | blob `4ccbfdd4b3ace6a74798999e13b4254fcb642131`; SHA-256 `f6ac1468ab37576f799512a01cc27bd285b43989e880b0b88a5770a5f9bdbd9b` | `verbatim-fragment`; second distinct `Bank` UUID/path and the same nested Document member. |
| `src/Subsystems/StandardSubsystems/StandardSubsystems.mdo` | `OneAgent_EDTproject/src/Subsystems/StandardSubsystems/StandardSubsystems.mdo:2-3,36` | blob `7971adb73a1728d66c2197a3cd5fa217ca0ae040`; SHA-256 `8da1bc8f4202042d0128cb1ab17201af089d81698ff57de55eb3d1586e0aac42` | `verbatim-fragment`; direct parent for the observed self-content case. |
| `src/Subsystems/StandardSubsystems/Subsystems/ObjectAttributesLock/ObjectAttributesLock.mdo` | `OneAgent_EDTproject/src/Subsystems/StandardSubsystems/Subsystems/ObjectAttributesLock/ObjectAttributesLock.mdo:2-3,14,21` | blob `81b1455b9d35248d72c5048921cb41a74391c038`; SHA-256 `d6a01a6f5cc75a6a82190811f97e042d60ba819bb18197d3fb858ef5d09106ea` | `verbatim-fragment`; exact deferred self-content token and qualified parent. |
| `src/CommonModules/AddressableStorage/AddressableStorage.mdo` | `OneAgent_EDTproject/src/CommonModules/AddressableStorage/AddressableStorage.mdo:2-3` | blob `30d1b9405b2e81e46f6ba50ea2745020f2ff89aa`; SHA-256 `fed03be21f4a4aa4fb08ea47fa50a8c854c7a680b2986d8d237bf1aa46865e08` | `verbatim-fragment`; exact shared metadata member root/UUID/name. |
| `src/Documents/BankAccountIssue/BankAccountIssue.mdo` | `OneAgent_EDTproject/src/Documents/BankAccountIssue/BankAccountIssue.mdo:2,10` | blob `79a8cd50d2a0cfbe270c7a18f410f053451c71eb`; SHA-256 `7e6badacf2c12a07bbd455491bc7c7f776f8b1b9b40f49ec4941491866501666` | `reduced-derived`; exact Document kind/UUID/name with unused namespaces and produced types removed. |
| `src/Configuration/Configuration.mdo` | No live-source field selected. | not applicable | `generated-scaffold`; supplies only a deterministic fixture project owner. |

## Complete fixture integrity

| Artifact | Fixture SHA-256 |
|---|---|
| `src/CommonModules/AddressableStorage/AddressableStorage.mdo` | `725f83330650e28a7ed5a44c664a498e4e68307bda1aede3e981c28ef4181738` |
| `src/Configuration/Configuration.mdo` | `ce5d31c3a15256328eddf01593436b391c3c093d962139b3ce3b5a87bb9291ca` |
| `src/Documents/BankAccountIssue/BankAccountIssue.mdo` | `e8c37ef011990041c260aca5bc6d42d738960458b52ea4eb424fd6e375080853` |
| `src/Subsystems/DNSCore/DNSCore.mdo` | `a4dab6477863a7fa6a782bcf4d7d9fa367e035b8c7fde51fdc81dfda611e907d` |
| `src/Subsystems/DNSCore/Subsystems/Common/Common.mdo` | `2bcdb6d1d92c28a5806f6ba204a30d87ce91f68ad0060e641f9e5e0648b68560` |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/FinancialAccounting.mdo` | `2deba0c08ed2f5fc71cbad14687cfb9861afd23d37a8f712569d9071bbf1f4e1` |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Treasury.mdo` | `c73e86bacbb8f77497eaec0e7ce5a91944497ec7e3300814c29110cc1b2d7d52` |
| `src/Subsystems/DNSCore/Subsystems/Common/Subsystems/FinancialAccounting/Subsystems/Treasury/Subsystems/Bank/Bank.mdo` | `3b41182377e13d4baa6d7744fb83f6f29c485807136fa94f322b391a612fc8cf` |
| `src/Subsystems/InterfaceFinancialAccounting/InterfaceFinancialAccounting.mdo` | `d0bc2abb358087914e292447af899510986d12ce6592c91e39d996d5ab97655a` |
| `src/Subsystems/InterfaceFinancialAccounting/Subsystems/Bank/Bank.mdo` | `121bb76983b80bc2f28fa5669d85a22ac46c646c0540d72e38e5276fe7f4230f` |
| `src/Subsystems/StandardSubsystems/StandardSubsystems.mdo` | `20c1dce8542b63e4524f2d05934bd9f65483512f76715749c3ed39c24e061750` |
| `src/Subsystems/StandardSubsystems/Subsystems/ObjectAttributesLock/ObjectAttributesLock.mdo` | `8c08992b44787e3a4c0159e45cdc8dfb5551494413223b37ee297bb6e8969d0b` |

## Deliberate omissions

The reduction omits unselected siblings and content, command-interface fields,
Subsystem aliases, broader metadata families, and semantic interpretation of
`Subsystem.<...>` content. Fatal hierarchy matrices and incremental source
transitions remain generated because contradictory or mutated live source is
not positive provenance evidence.
