# Writes Static EDT Evidence Project

This directory is the canonical positive source project for the first Writes
slice accepted by ADR-0022. It contains one generated Configuration, one reduced
real Document descriptor and Object Module, and two reduced real Accumulation
Register descriptors. It is fixture evidence only: no Rust test or production
path consumes this project yet.

## Project contract

The generated Configuration uses the deterministic fixture-only UUID
`50000000-0000-0000-0000-000000000000` and name `WritesFixture`. Neither
value is real-source evidence.

The positive candidate order is fixed by Object Module source order:

1. fixture line 2: `RegisterRecords.CashAccountBalance.Write();`
2. fixture line 3: `RegisterRecords.RefundBankPayment.Write();`

Using the expected deterministic key `(fixture line, normalized register
name)`, the order is `(2, cashaccountbalance)` followed by
`(3, refundbankpayment)`.

The Object Module is shared with future BSL candidate tests and EDT integration
tests. No duplicate positive BSL fixture exists. The owning Document contains
exactly the two selected declarations in the same order, and the project
contains exactly the two matching Accumulation Register descriptors.

## Normalization and hashing

Source ranges are inclusive and one-based. To reproduce a normalized source
fragment, read the source file as UTF-8, select the stated physical lines, remove
a UTF-8 BOM if it occurs at the beginning of the selected bytes, convert CRLF or
CR line endings to LF, remove trailing ASCII space and tab bytes from every
selected line, preserve leading whitespace, internal bytes, blank lines, and
line order, remove extra terminal line endings, and append exactly one LF. The
recorded fragment SHA-256 is the digest of those normalized UTF-8 bytes.

Generated scaffold is hashed as exact fixture UTF-8 bytes with LF endings and
exactly one terminal LF. Every source blob ID is the current result of
`git hash-object <source-path>`.

## Artifact manifest

Artifacts are ordered lexically by repository-relative filename.

| Artifact and fixture lines | Source origin and blob | Normalized fragment SHA-256 | Treatment, context, and layer | Expected first-slice role and future typed outcome |
|---|---|---|---|---|
| `src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo:2` | `OneAgent_EDTproject/src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo:2`; blob `a37e9f8214ca0e662c037daf4a50e27d59da6d31` | `ccc36398604c19b3bb4c613473325da081e5060c60c5dc96a6195eb65b1bf354` | `verbatim`; real Accumulation Register root kind and UUID; EDT integration only | Proves `AccumulationRegister` kind and UUID `ac997c18-b62c-4bc3-9079-9a729ad5253c`. Future outcome: one compatible target descriptor participates in exact resolution. |
| `src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo:3` | `OneAgent_EDTproject/src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo:11`; blob `a37e9f8214ca0e662c037daf4a50e27d59da6d31` | `f08731682c68d0cdfc0b02cbadc680c5e23d79f13e7387e3b802f815dd525cb5` | `verbatim`; exact target name; EDT integration only | Proves exact name `CashAccountBalance`. Future outcome: unique compatible target when combined with the declaration and graph node. |
| `src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo:2` | `OneAgent_EDTproject/src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo:2`; blob `7a40377240c8d7608032895fde7fda5cb65a285a` | `12f68f5cdd4af1cde9acc2569460b0643f956eecdffac6f1670c5cf83202e014` | `verbatim`; real Accumulation Register root kind and UUID; EDT integration only | Proves `AccumulationRegister` kind and UUID `f014a53e-bf0e-4dc4-9a8c-93ef663d9108`. Future outcome: one compatible target descriptor participates in exact resolution. |
| `src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo:3` | `OneAgent_EDTproject/src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo:11`; blob `7a40377240c8d7608032895fde7fda5cb65a285a` | `f2d48737bc73f19c2076a4ada5d008cb49171cfda87c3f216a52cf240915515b` | `verbatim`; exact target name; EDT integration only | Proves exact name `RefundBankPayment`. Future outcome: unique compatible target when combined with the declaration and graph node. |
| `src/Configuration/Configuration.mdo:1-4` | Generated fixture-only content; no source blob | `90096aa2eaf4514cd05c2fb3c1c711e7eee3a5aaaae2c7bfdb4cd2cdda99b9fb` | `generated-scaffold`; minimal Configuration following the Reads project convention; EDT integration only | Makes the fixture tree structurally loadable. It supplies no Writes evidence and has no candidate or graph outcome. |
| `src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:2` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:76`; blob `7becbe8d31387d9670fb74dad8dd6ac695d83cbc` | `f555f64b52b9afc83fdf01e59b4e7bd77d6f0facf368f4d56502596c66ca057b` | `verbatim`; Document Object Module owned by `RefundOfPaymentByOrder`, generated Procedure `Posting` wrapper; shared by BSL and EDT integration | Complete zero-argument candidate for `CashAccountBalance`. Future outcome: later exact declaration and target resolution may produce one Writes edge. |
| `src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:3` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:78`; blob `7becbe8d31387d9670fb74dad8dd6ac695d83cbc` | `123a001bb931739041b5a5e1533efc3efdd07d5df107c593ce263330cdf93a3c` | `verbatim`; same owner and generated Procedure wrapper; shared by BSL and EDT integration | Complete zero-argument candidate for `RefundBankPayment`. Future outcome: later exact declaration and target resolution may produce one Writes edge. |
| `src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:2` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:2`; blob `eb966a37296ab084f511a118da6d6fcf7d2d6a84` | `61cbe99247f4bcb8339381c896aed6ff0728b2a68cb22f1216630dd3461a4ee8` | `verbatim`; real Document root kind and UUID; EDT integration only | Proves Document ownership and UUID `ed647f67-f8fe-476b-8823-8d52b365ab20`. Future outcome: supplies the owning typed Document descriptor. |
| `src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:3` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:10`; blob `eb966a37296ab084f511a118da6d6fcf7d2d6a84` | `9f0ba4bd3203eb4e75a22148a36de535f7fc183486c153e1fd47c39dc20607ba` | `verbatim`; exact owning Document name; EDT integration only | Proves exact name `RefundOfPaymentByOrder`. Future outcome: preserves owner identity for declaration matching. |
| `src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:4` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:79`; blob `eb966a37296ab084f511a118da6d6fcf7d2d6a84` | `cb956816e8932961a8344c19550d356faf053bbf91a3558f3cf63eb670131d10` | `verbatim`; selected owning Document declaration; EDT integration only | Proves `AccumulationRegister.CashAccountBalance`. Future outcome: one compatible typed declaration for the first candidate. |
| `src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:5` | `OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo:81`; blob `eb966a37296ab084f511a118da6d6fcf7d2d6a84` | `b8d3acdccfce366478556bafbe4c740c14edc1aab74472f90796749fc9016294` | `verbatim`; selected owning Document declaration; EDT integration only | Proves `AccumulationRegister.RefundBankPayment`. Future outcome: one compatible typed declaration for the second candidate. |

## Reduction and scaffold map

The Configuration descriptor is entirely generated scaffold. In
`ObjectModule.bsl`, lines 1 and 4 are the generated minimal `Posting`
Procedure wrapper; only lines 2 and 3 are verbatim source evidence. In the
Document descriptor, lines 1 and 6 are generated XML scaffold; lines 2-5 are
the verbatim selected evidence. In each target descriptor, lines 1 and 4 are
generated XML scaffold; lines 2-3 are verbatim selected evidence.

Unrelated descriptor properties, produced types, synonyms, members, and
unselected Document register declarations were removed. No source excerpt was
made non-contiguous: each retained source line is recorded as its own verbatim
fragment. The reduced descriptors preserve the real root kinds, real UUIDs,
exact names, and exactly the two selected `registerRecords` values.

For complete-artifact integrity, the exact fixture SHA-256 values are:

| Artifact | Fixture SHA-256 |
|---|---|
| `src/AccumulationRegisters/CashAccountBalance/CashAccountBalance.mdo` | `281845e55aca3a1b121c771b46c86ff96de497b325a47b5c329ae1c56d842e1c` |
| `src/AccumulationRegisters/RefundBankPayment/RefundBankPayment.mdo` | `b6fefe78256e859a33c05be5b6d2cea61c8b5d00ce8275639591931542332cc9` |
| `src/Configuration/Configuration.mdo` | `90096aa2eaf4514cd05c2fb3c1c711e7eee3a5aaaae2c7bfdb4cd2cdda99b9fb` |
| `src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl` | `5329d0a007529ca21d6103d54d1cb2cebc1c1a53e69a479a9e168e230f6b9f09` |
| `src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo` | `54d82b4f8e678138c790b739dfd3c32c98b3417922fc77880df67f1beae6cc0c` |

## Deliberate omissions

No malformed, missing, ambiguous, incompatible, duplicate, partial-workspace,
wrong-kind, or conflicting declaration/target project was invented. Those
states belong in generated Rust tests after typed declaration and resolution
models exist. The Unknown syntax categories listed in the BSL corpus manifest
are also absent from this project.
