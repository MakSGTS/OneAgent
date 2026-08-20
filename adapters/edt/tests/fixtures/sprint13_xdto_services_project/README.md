# Sprint 13 XDTO and service production fixture

This self-contained EDT project is a source-preserving reduction of the ignored
`OneAgent_EDTproject/src/` corpus. It is tracked so positive production evidence
does not depend on ignored files. UUIDs, names, namespaces, direct XDTO kind,
service hierarchy, optional values, type declarations, directions, handler
names, and BSL declaration kinds are copied from the listed live artifacts.
Unselected synonyms, comments, unrelated siblings, nested XDTO properties, and
BSL bodies are omitted.

Reduction contract: every row is `reduced-derived`; selected semantic values are
verbatim, while surrounding source is minimized to the smallest valid EDT
artifact. The large Enterprise Data schema is deliberately represented by two
Value and two Object declarations rather than copying its 1,253,483-byte body.

## Source and reduced artifact integrity

| Reduced artifact | Live source under `OneAgent_EDTproject/src/` | Live SHA-256 | Reduced SHA-256 |
|---|---|---|---|
| `src/Configuration/Configuration.mdo` | `Configuration/Configuration.mdo` | `017f5f4efeef37d63b72884d71a6770696763200c82eea3f2e0f38c634d3950c` | `aa1258a328ce01b4c149ad2e2d52eb7243da125b15599dfdec4048dc4003977d` |
| `src/XDTOPackages/CurrencyRates/CurrencyRates.mdo` | `XDTOPackages/CurrencyRates/CurrencyRates.mdo` | `e206ad8ee8c6a85f03671041aa72205f44b52fc4951a0fab49471fb4f452a280` | `4f01720f5388eb4510e82d79420954a52f2ebf6654b76c7c79c28ff0c6cd41af` |
| `src/XDTOPackages/CurrencyRates/Package.xdto` | `XDTOPackages/CurrencyRates/Package.xdto` | `e8fbc57e9f2213165a7c4feb77d8834ef547f475beeecc3155d8a4bfff38f220` | `3886f884a6f7b349147fbc1dc9554e3f88f57acffd474d5bc90b8368f96feddc` |
| `src/XDTOPackages/ApplicationExtensionsManifest_1_0_0_1/ApplicationExtensionsManifest_1_0_0_1.mdo` | `XDTOPackages/ApplicationExtensionsManifest_1_0_0_1/ApplicationExtensionsManifest_1_0_0_1.mdo` | `58b707df8dbbacb9ccf6dd762c29a2bf993406364f70f1af290aee0be5082002` | `e826e00706bd181c2f9d2b37e11126c55a062359aadac5cb7edfc51465ebc16c` |
| `src/XDTOPackages/ApplicationExtensionsManifest_1_0_0_1/Package.xdto` | `XDTOPackages/ApplicationExtensionsManifest_1_0_0_1/Package.xdto` | `6d53a376949d42871077a3614fcc82c6e1d7105bcb6ad977d429c9a3d540a4e4` | `c4c81c7b05a4027b21b46893005081178071ac41df0fd1a7dd2a48d693f672a6` |
| `src/XDTOPackages/EnterpriseData_1_17_3/EnterpriseData_1_17_3.mdo` | `XDTOPackages/EnterpriseData_1_17_3/EnterpriseData_1_17_3.mdo` | `e894948fe1396910f99584be75e95314ae216afb7ad4dd06a7ca0831586a0229` | `a7b7bc3e4c2df0e0c86ac9fd54ec16c5f833ee387612e47c13883a8260318116` |
| `src/XDTOPackages/EnterpriseData_1_17_3/Package.xdto` | `XDTOPackages/EnterpriseData_1_17_3/Package.xdto` | `130b3e226c44223099b75c73ac4d647d8e07249cef34c9c163d2acec506e4f10` | `094d08c0958adc0c6977d77cdd64f77e7a8340c0c071049ecf841dc0b490215a` |
| `src/XDTOPackages/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | `XDTOPackages/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | `dbb883f1be02bd75f2812e88ccd674ce53f2ef74f298472e38011741c5bc86dc` | `1ea0d41cd1db1788e05f38569731b3b49fe6ceb1f881c7cc35b5f95479732780` |
| `src/XDTOPackages/EnterpriseDataExchange_1_0_1_1/Package.xdto` | `XDTOPackages/EnterpriseDataExchange_1_0_1_1/Package.xdto` | `70d5efefad5fad76b65c5ddc5c126b970d1808e1651e4a8ca62daea79845921a` | `c81633094ee9d310ba14ab8754946661d3e9441c287b243113e2bc55caaa394a` |
| `src/HTTPServices/Site/Site.mdo` | `HTTPServices/Site/Site.mdo` | `5dd1589f54f3b57f50ea38040669c17a99fc3a97bde496971794b5d70768f299` | `300cdf6073926b82e0d2cc6b9fc5afe40a49f57c499973d903c584ddcad665ae` |
| `src/HTTPServices/Site/Module.bsl` | `HTTPServices/Site/Module.bsl` | `c6d73a7da06f3e0b2c9509b5170004c0f5f5873e4a6a5aef6a4ac859e058f0d6` | `b11c22e45f99a20857949e23f98809a60620b1dc024a17055d3e0ff40a832e2b` |
| `src/WebServices/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | `WebServices/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | `6bb5c9b64aeb23816206652524ad44b5355fc0f86a144bfb114dbf88d40f5777` | `80515b51485aa1c0d5b9835010fde0894477d2a166be346e2f2297953bd29155` |
| `src/WebServices/EnterpriseDataExchange_1_0_1_1/Module.bsl` | `WebServices/EnterpriseDataExchange_1_0_1_1/Module.bsl` | `2223979eb4335cc97face0c43b042625ffa52cb53f59d06241cdc7de1005553c` | `8fbc656ad17df95890d9efa26830dbf3e7e006391ddf14e4ad482634bca1c4e1` |
| `src/WebServices/InterfaceVersion/InterfaceVersion.mdo` | `WebServices/InterfaceVersion/InterfaceVersion.mdo` | `3649666fc86aa0d73b65e8d5c4bc36e2f6eae07ec3f428da3663724041c9c221` | `e91748ffaacd1eceaca99aa71cd89a8a5124ab91c7e5c49bf714f4f36a9798a6` |
| `src/WebServices/InterfaceVersion/Module.bsl` | `WebServices/InterfaceVersion/Module.bsl` | `e42f8e36bc6c09087a195258309366af9a643f7691bc9afbf5b66bb0181d1a1b` | `5c569f3f8d10f2848cb79db541ca6227d70dcaccc51e86b89bcf7ac8fa42299d` |
| `src/WebServices/Exchange/Exchange.mdo` | `WebServices/Exchange/Exchange.mdo` | `7842c62f21a6623aa1c25b8755124c0f2df8bf62ff086d2f7177e22291662cbc` | `a18804099ca98ba886950e6b7e0ca162788ea22162d0e3ce79ab2152210b9fe2` |
| `src/WebServices/Exchange/Module.bsl` | `WebServices/Exchange/Module.bsl` | `27acb605034d7a6d5fdf7663d35b2da1b74e2ac61d814cdc0432b83f7364d191` | `3b3799d114bade03aed35baac2f693861a1df3698c54d7f03fe2965e8070ca6e` |

## Preserved evidence matrix

- small object-only (`CurrencyRates`), mixed Value/Object
  (`ApplicationExtensionsManifest_1_0_0_1`), and reduced large Value/Object
  (`EnterpriseData_1_17_3`) direct-type shapes;
- the repository-owned `PrepareDataOperationResult` type used by the internal
  Web return declaration;
- explicit POST and absent GET `httpMethod` values with exact owned Functions;
- internal ReferenceValue, external StringValue, and absent Web package forms;
- internal and external return/value types, absent/`Out`/`InOut` directions,
  nillability, UUID identities, immediate hierarchy, and exact owned Functions.

Nested XDTO properties/imports/restrictions, external schema nodes, routes,
transport/WSDL/runtime behavior, and Designer XML remain deliberately absent.
