# Paired Designer XML module parser fixture

The fixture retains exact byte-for-byte copies of one paired Common Module:

| Source | Registered path | Tracked path | Raw SHA-256 | Bytes |
|---|---|---|---|---:|
| Designer | `OneAgent_DesignerXML/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl` | `designer/DynamicSecurityOverridable.bsl` | `b798303db6df6427ac5e14abd616cf0838254e0262c22585b033950bb7642e48` | 154 |
| EDT | `OneAgent_EDTproject/src/CommonModules/DynamicSecurityOverridable/Module.bsl` | `edt/DynamicSecurityOverridable.bsl` | `b56a39eedd53b8f621421e7e17dd59781ef3b6769e61f0e8b89c4192a7dac184` | 141 |

Reduction: none. The only accepted comparison normalization removes one leading
UTF-8 BOM and converts CRLF or bare CR line endings to LF. The normalized texts
are byte-equal. Generated negative fixtures in unit tests are explicit parser
mutations and are not registered source vocabulary or conformance evidence.
