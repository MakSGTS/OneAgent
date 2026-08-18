# Writes Source Evidence Fixtures

This directory contains repository-backed BSL boundary evidence for the first
Writes slice accepted by ADR-0022. The files are evidence assets only: no Rust
parser or test consumes this corpus yet. The canonical positive input is shared
with EDT integration at
`adapters/edt/tests/fixtures/writes_project/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl`;
it is intentionally not duplicated here.

## Normalization and hashing

Every source range is inclusive and one-based. To reproduce a normalized
fragment, read the source file as UTF-8, select the stated physical lines, remove
a UTF-8 BOM if it occurs at the beginning of the selected bytes, convert CRLF or
CR line endings to LF, remove trailing ASCII space and tab bytes from every
selected line, preserve leading whitespace, internal bytes, blank lines, and
line order, remove extra terminal line endings, and append exactly one LF. The
recorded SHA-256 is the digest of those normalized UTF-8 bytes. Each fixture is
exactly its normalized fragment.

Every source blob ID is the current result of `git hash-object <source-path>`.
All fragments are contiguous bounded excerpts. Removing trailing horizontal
whitespace is the only content reduction inside the stated ranges.

## Manifest

Entries are ordered lexically by fixture filename.

| Fixture and fixture lines | Source origin and blob | Normalized fragment SHA-256 | Treatment, context, and layer | Expected first-slice classification and future typed outcome |
|---|---|---|---|---|
| `aliased_register_record_set.bsl:1-37` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:132-168`; blob `7becbe8d31387d9670fb74dad8dd6ac695d83cbc` | `ff8c923e74c687f592e7cedcdc50d3a748fac2d12b903a5a07899fa2238aed37` | `bounded-excerpt`; Document Object Module owned by `RefundOfPaymentByOrder`, Procedure `CreateOrderExchangesRecords`; BSL-only | Requires alias/value flow; no first-slice candidate eligible for resolution. Future outcome: typed requires-value-flow or unsupported-receiver result; no edge. |
| `archive_file_write.bsl:1-22` | `OneAgent_EDTproject/src/WebServices/Exchange/Module.bsl:238-259`; blob `1007cac6dfd8c590156cb33dd536dbc89c9d5a2d` | `cbcc75edf24721a5d4312a30fd269e132abbf8130d63d53bccf9068e8458f115` | `bounded-excerpt`; Web Service module, Function `PrepareGetFile`; BSL-only | Non-persistent archive effect. Future outcome: statically non-persistent receiver; no edge. |
| `argument_bearing_information_register.bsl:1-18` | `OneAgent_EDTproject/src/Documents/CorrectingProductStatus/ObjectModule.bsl:530-547`; blob `6b2919a643f7bd03ebf222b0d47594b7cf496e59` | `5f9a1deeaa9677f515672c44b2e5074e1a8a2a1f8588d8a9e8eebda2716aa394` | `bounded-excerpt`; Document Object Module owned by `CorrectingProductStatus`, Procedure `CreateHistoryOfCustomsClearanceStatusesRecords`; BSL-only | Recognized write-like statement with a non-empty argument and deferred Information Register family. Future outcome: typed unsupported-non-empty-arguments result; no edge. |
| `async_scope_write.bsl:1-38` | `OneAgent_EDTproject/src/InformationRegisters/CS_Queue/Forms/FileManagementForm/Module.bsl:214-251`; blob `5b44a981ea8046c246c94eb25d42b16cb6f2b1b9` | `e350a5138b4ca2533f951b0a15bc77977d98e382471dbba3aa93b04a2a2811d4` | `bounded-excerpt`; Information Register form module owned by `CS_Queue`, async Procedure `SaveCurrentFile`; BSL-only | The current generic extractor has no containing scope. Future outcome: typed missing/unsupported-scope result unless async declarations become an explicit prerequisite; no edge. |
| `binary_file_write.bsl:1-10` | `OneAgent_EDTproject/src/CommonForms/EditSpreadsheetDocument/Module.bsl:91-100`; blob `07e32d98f5bb882be8d6d88426aeb76985cd2768` | `a8823000789c51cf568364bf17375d32053d05d084f94b1ac835359369a3c96c` | `bounded-excerpt`; Common Form module owned by `EditSpreadsheetDocument`, Procedure `OnCreateAtServer`; BSL-only | Non-persistent binary file effect. Future outcome: statically non-persistent or unsupported receiver; no edge. |
| `chained_common_module_receiver.bsl:1-22` | `OneAgent_EDTproject/src/CommonModules/GuaranteeIntegration/Module.bsl:2925-2946`; blob `a4f6a715871c9b9abfcf71e518f918e330a07513` | `4f10839a53e438c5fafcdc9fa71ab50631d300df062aa598539e782214ef4167` | `bounded-excerpt`; Common Module `GuaranteeIntegration`, Procedure `ProductsInStorageBinsRecords`; BSL-only | Chained receiver plus unsupported metadata owner. Future outcome: typed unsupported-receiver-shape and/or unsupported-module-owner result; no edge. |
| `chained_manager_receiver.bsl:1-11` | `OneAgent_EDTproject/src/Documents/ServiceSale/ManagerModule.bsl:150-160`; blob `59e9609b1b55e5adff0b2eccfe22df334775039c` | `ad3634577adc59e3b7982fc6cd7440f7d98411f8235634c015e44b5acf6549ea` | `bounded-excerpt`; Document Manager Module owned by `ServiceSale`, Procedure `ConvertServiceSalesOrders`; BSL-only | Chained receiver plus unsupported module owner. Future outcome: typed unsupported-receiver-shape and/or unsupported-module-owner result; no edge. |
| `collection_level_write.bsl:1-18` | `OneAgent_EDTproject/src/Documents/TransferToAssets/ObjectModule.bsl:65-82`; blob `34504faeb49e5a438fce45f2561a29a08d7fcaeb` | `0dfc90116efa1e793f69e63ac9673d581740b6764cae55db329efafb272c2bc5` | `bounded-excerpt`; Document Object Module owned by `TransferToAssets`, Procedure `Posting`; BSL-only | Collection-level write has no single named target. Future outcome: typed unsupported-write-shape result; no edge. |
| `comment_only_write.bsl:1-6` | `OneAgent_EDTproject/src/CommonModules/FilesOperations/Module.bsl:28-33`; blob `9bf03ff0ae8a65d5b3c96dcd7d855d7ed157fcd2` | `1016351aa1a6bae1798964bc383f487364b049bd940a1ae0d9b9fc135f23f9d3` | `bounded-excerpt`; Common Module `FilesOperations`, documentation example outside callable scope; BSL-only | Comment text is not a call. Future outcome: no candidate and no diagnostic. |
| `computed_receiver_write.bsl:1-11` | `OneAgent_EDTproject/src/CommonModules/CurrencyRateOperationsInternal/Module.bsl:98-108`; blob `3aae82a135f52bc68626d209d7db2b5f8888ed09` | `d0120e19e783e2af26df547b25e6247991c458f6ffb36fafe5245dbc6ee2b2cd` | `bounded-excerpt`; Common Module `CurrencyRateOperationsInternal`, Procedure `CopyCurrencyRates`; BSL-only | Computed receiver with arguments is outside the accepted receiver shape. Future outcome: typed dynamic-target or unsupported-receiver-shape result; no edge. |
| `external_input_file_write.bsl:1-20` | `OneAgent_EDTproject/src/WebServices/Exchange/Module.bsl:320-340`; blob `1007cac6dfd8c590156cb33dd536dbc89c9d5a2d` | `46e7a2b7bfb623c18bcc76040db38066c9c11d967c3da0fe2ab1bfab3790aec1` | `bounded-excerpt`; Web Service module, Function `PutFilePart`; BSL-only | External-input file effect, not persistent metadata. Future outcome: non-persistent or unresolved external receiver; no edge. |
| `local_document_value_flow.bsl:1-18` | `OneAgent_EDTproject/src/DataProcessors/WorkplaceForSales/Forms/Form/Module.bsl:901-918`; blob `c66133be8890d527853501825dc1ae9f1ec2115d` | `3eee22d6d5b5dc7031df5a967b05732e04b55d522ec4055585e2438d69f26076` | `bounded-excerpt`; Data Processor form module owned by `WorkplaceForSales`, Procedure `CreateProductSale`; BSL-only | Local Document object operation requires value/type flow. Future outcome: typed requires-value-flow or unresolved-receiver result; no edge. |
| `local_predefined_item_value_flow.bsl:1-21` | `OneAgent_EDTproject/src/ChartsOfCharacteristicTypes/DetailTypesOfNamedProducts/ManagerModule.bsl:229-249`; blob `40ff5c82847aff4854678cd41d3acd88574ddefc` | `dac678035c1782ab41808adbd51d92fb5d52bf38efae01d0dff8235e71fa94e6` | `bounded-excerpt`; Chart of Characteristic Types Manager Module owned by `DetailTypesOfNamedProducts`, Procedure `AddPredefined_Yoda`; BSL-only | Local predefined object operation requires value/type flow. Future outcome: typed requires-value-flow or unresolved-receiver result; no edge. |
| `property_assignment_and_call.bsl:1-14` | `OneAgent_EDTproject/src/Documents/ProductReturnFromMarkdown/ObjectModule.bsl:90-103`; blob `b19bf1863ba798f3d5219984e177e4f6afa179bf` | `87d900652983afccecba84d444d3413db3bba4e4a7ce200094e7bd0a6fd802ee` | `bounded-excerpt`; Document Object Module owned by `ProductReturnFromMarkdown`, Procedure `Posting`; BSL-only | The property assignment produces no candidate; the later zero-argument invocation is an independent complete candidate. Future outcome: no candidate for fixture line 11 and a complete candidate for fixture line 13. |
| `text_file_write.bsl:1-11` | `OneAgent_EDTproject/src/CommonForms/CheckUpdateFile/Module.bsl:60-70`; blob `15df02249e8952fdbee66d46491e9d5b1bad6b48` | `97f85dfbc052525018c94b6b4b7bc3698c8b9fafe0a3bf651e8be00c5f2fa1de` | `bounded-excerpt`; Common Form module owned by `CheckUpdateFile`, Function `OnlyBuildNumberOfMainConfigurationChanged`; BSL-only | Non-persistent text/file effect. Future outcome: statically non-persistent receiver; no edge. |
| `ui_form_write.bsl:1-12` | `OneAgent_EDTproject/src/Reports/FinancialReport/Commands/CheckReportKind/CommandModule.bsl:35-46`; blob `b849fad660660d12a4b52d6f3beca9e6669d58ef` | `f97bfd718f17e08e46ecbf47e31c4744cffbd9ef7eac8b53bc2709d5bfd66fc1` | `bounded-excerpt`; Report command module owned by `FinancialReport.CheckReportKind`, Procedure `CommandProcessingCompletion`; BSL-only | UI form behavior. Future outcome: typed unsupported UI receiver; no edge. |

## Deliberate evidence gaps

No fixture is present for a string payload containing `.Write(`, malformed or
incomplete statements, localized receiver or method spelling, language-level
alias syntax, a general external-component `.Write(...)`, persistent dynamic
member access, or malformed, duplicate, or conflicting Document declarations.
Those cases are Unknown in the repository-owned source corpus.

Missing, ambiguous, incompatible, duplicate, partial-workspace, and wrong-kind
resolver states are also absent. They are accepted future typed states, not real
source syntax, and belong in generated tests after the declaration and
resolution models exist.
