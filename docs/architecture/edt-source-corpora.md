# Local EDT Source Corpora

## Status

Repository-local inventory for ignored EDT projects used as architecture and
fixture-derivation evidence. These projects are not tracked test fixtures and
must not become CI prerequisites.

## Corpus policy

- Keep complete EDT workspaces ignored because they contain large generated
  workspace state and source trees that are unsuitable for Git history.
- Treat corpus observations as point-in-time research evidence, not permanent
  runtime constants or accepted source-independent semantics.
- Derive only the smallest necessary tracked fixtures, with exact source paths,
  hashes, reduction treatment, and expected behavior.
- Do not broaden parsers, graph contracts, Coverage, or validation merely
  because a new serialized shape is present. Investigate and accept the shape
  through the normal architecture workflow first.
- Do not make tests depend on either local corpus being installed.

## Inventory

Inventory observed on 2026-08-20:

| Corpus | EDT project root | Source size | Source files | Top-level families | Role |
|---|---|---:|---:|---:|---|
| OneAgent | `OneAgent_EDTproject` | 252 MB | 9,658 | 39 | Existing primary planning and fixture-provenance corpus |
| Retail | `Retail_edt_project/Розница_базовая` | 3.1 GB | 53,652 | 43 | Additional broad Russian-language and Unicode research corpus |

The complete Retail EDT workspace occupies 7.7 GB and contains 75,247 files,
including Eclipse/EDT `.metadata` state. Only the project subtree under
`Розница_базовая/src/` is source evidence.

Selected source-family comparison:

| Family or artifact | OneAgent | Retail |
|---|---:|---:|
| BSL modules | 2,719 | 15,181 |
| MDO descriptors | 3,635 | 17,318 |
| XDTO artifacts / package directories | 20 | 411 |
| DCS artifacts | 72 | 491 |
| HTTP Services | 2 | 18 |
| Web Services | 8 | 18 |
| Event Subscriptions | 99 | 449 |
| Reports | 56 | 353 |
| Subsystems | 13 | 65 |
| Catalogs | 116 | 729 |
| Documents | 81 | 344 |
| Information Registers | 174 | 1,151 |
| Accumulation Registers | 21 | 137 |
| Common Modules | 509 | 3,316 |
| Common Forms | 84 | 391 |

The Retail project has configuration UUID
`de29c81d-d880-419b-8f65-cc49813f0e9a`, exact name `РозницаБазовая`, Russian
script variant, and configuration version `3.0.13.251`. Its paths and
identifiers exercise Cyrillic text and decomposed Unicode filesystem spellings
that are underrepresented in the existing primary corpus.

## Current compatibility boundary

A diagnostic production build was run with
`FileSystemEdtSemanticGraphBuilder` against
`Retail_edt_project/Розница_базовая`. Discovery reached Web Service parsing and
then returned:

```text
ServiceDescriptor(DuplicateField {
    context: "mdclass:WebService",
    field: "xdtoPackages",
})
```

This is useful evidence of a current parser boundary rather than proof that the
Retail project is malformed. Its 18 Web Service descriptors have this direct
`xdtoPackages` cardinality distribution:

| Cardinality | Web Services |
|---:|---:|
| 0 | 3 |
| 1 | 13 |
| 2 | 1 |
| 4 | 1 |

`EquipmentService` declares four repository XDTO packages. `MobileService`
declares one repository package and one external namespace. The accepted
[corrective source investigation](web-service-xdto-packages-source-investigation.md)
and amended [ADR-0035](../adr/0035-xdto-service-semantics.md) confirm direct
`xdtoPackages` cardinality as zero-or-more, define canonical collection and
request semantics, and preserve complete-snapshot namespace/type resolution.
The current production parser still accepts only the zero-or-one shape proven
by the original corpus and classifies a repeated field as fatal. A bounded
implementation task remains required before Sprint 14 begins.

Until that corrective implementation and its validation gate complete, the
Retail project is suitable for:

- source-shape investigation;
- Unicode and Russian-identifier audits;
- identifying representative future fixtures;
- checking whether accepted contracts generalize to a broader configuration.

It is not yet suitable for:

- a successful whole-project EDT builder gate;
- a required local or CI test dependency;
- direct Coverage promotion;
- copying large source subtrees into tracked fixtures.

Historical investigation documents retain their original explicitly named
corpus and baseline. Future investigations must state whether they inspect
OneAgent, Retail, or both, and must keep counts separated by corpus.
